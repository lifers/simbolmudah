#include "pch.hpp"
#include "App.xaml.h"
#include "MainWindow.xaml.h"
#include "AppManager.h"
#include <winrt/in_app_tutorial.h>
#include <wil/cppwinrt_helpers.h>


namespace winrt::simbolmudah_ui::implementation
{
    using namespace in_app_tutorial;
    using namespace LibSimbolMudah;
    using namespace Microsoft::UI;
    using namespace Dispatching;
    using namespace Xaml;
    using namespace Data;
    using namespace Windows;
    using namespace Foundation;
    using namespace Storage;

    /// <summary>
    /// Initializes the singleton application object.  This is the first line of authored code
    /// executed, and as such is the logical equivalent of main() or WinMain().
    /// </summary>
    App::App() :
        mainThread{ DispatcherQueue::GetForCurrentThread() },
        sequenceDefinition{}, keyboardTranslator{ this->sequenceDefinition },
        appManager{ ApplicationData::Current().LocalSettings() },
        settingsChangedRevoker{ appManager.PropertyChanged(auto_revoke, { this->get_weak(), &App::OnSettingsChanged }) }
    {
        // Do not close the application when the last window is closed.
        this->DispatcherShutdownMode(DispatcherShutdownMode::OnExplicitShutdown);

        // Load the tray icon path.
        StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Images/favicon.ico")).Completed({
            this->get_weak(), &App::OnNotifyIconPathInitialized
        });

        // Build the keyboard translator finite state automaton.
        this->buildProgress = this->RebuildDefinition();

#if defined _DEBUG && !defined DISABLE_XAML_GENERATED_BREAK_ON_UNHANDLED_EXCEPTION
        this->UnhandledException([](IInspectable const&, UnhandledExceptionEventArgs const& e)
        {
            if (::IsDebuggerPresent())
            {
                const auto errorMessage{ e.Message() };
                ::__debugbreak();
            }
        });
#endif
    }
    
    void App::SwitchPopupWindow(bool state)
    {
        if (state)
        {
            if (!this->popupWindow)
            {
                this->popupWindow = simbolmudah_ui::PopupWindow{
                    this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition };
            }
        }
        else
        {
            if (this->popupWindow)
            {
                this->popupWindow.Close();
                this->popupWindow = nullptr;
            }
        }
    }

    void App::SwitchKeyboardHook(bool state)
    {
        if (state)
        {
            if (!this->keyboardHook)
            {
                this->keyboardHook = KeyboardHook{ this->keyboardTranslator };
                if (this->appManager.UseHookPopup()) { this->SwitchPopupWindow(true); }
            }
        }
        else
        {
            this->SwitchPopupWindow(false);
            this->keyboardHook = nullptr;
        }

        if (this->notifyIcon) { this->notifyIcon.GetHookEnabled(state); }
    }

    /// <summary>
    /// Invoked when the application is launched.
    /// </summary>
    void App::OnLaunched(LaunchActivatedEventArgs const&)
    {
        // Initialize the tutorial dialog.
        TutorialDialog::Initialize(this->Resources(), [weak{ this->get_weak() }](auto&&, bool state) {
            if (const auto self{ weak.get() }; self) {
                self->SwitchKeyboardHook(state);
                self->SwitchPopupWindow(state);
            }
        });
        const auto& dialog{ TutorialDialog::GetDialog() };
        this->tutorialOpenedToken = dialog.Opened(auto_revoke, [weak{ this->get_weak() }](auto&&, auto&&) {
            // Temporarily disable the keyboard hook.
            if (const auto self{ weak.get() }; self) { self->SwitchKeyboardHook(false); }
        });
        this->tutorialClosingToken = dialog.Closing(auto_revoke, [weak{ this->get_weak() }](auto&&, auto&&) {
            // Re-enable the keyboard hook according to current settings.
            if (const auto self{ weak.get() }; self && self->appManager.HookEnabled())
            {
                self->SwitchKeyboardHook(true);
            }
        });

        if (this->appManager.MainWindowOpened())
        {
            this->OpenWindow();
        }

        this->OnSettingsChanged(nullptr, nullptr);
    }

    /// <summary>
    /// Builds the keyboard translator finite state automaton.
    /// </summary>
    IAsyncAction App::RebuildDefinition()
    {
        co_await resume_background();
        const auto packageLocation{ ApplicationModel::Package::Current().InstalledLocation().Path() };
        const auto keysymdefPath{ packageLocation + L"\\Assets\\Resources\\keysymdef.h.br" };
        const auto composedefPath{ packageLocation + L"\\Assets\\Resources\\Compose.pre.br" };
        const auto annotationPath{ packageLocation + L"\\Assets\\Annotations" };
        this->sequenceDefinition.Rebuild(keysymdefPath, composedefPath, annotationPath);
    }

    /// <summary>
    /// Callback for when the settings change.
    /// </summary>
    void App::OnSettingsChanged(IInspectable const&, PropertyChangedEventArgs const&)
    {
        this->SwitchKeyboardHook(this->appManager.HookEnabled());
        this->SwitchPopupWindow(this->appManager.UseHookPopup() && this->appManager.HookEnabled());
        this->SwitchNotifyIcon(this->appManager.NotifyIconEnabled());
    }

    fire_and_forget App::OnNotifyIconPathInitialized(IAsyncOperation<StorageFile> const& op, AsyncStatus)
    {
        const auto filepath{ op.GetResults().Path() };
        co_await wil::resume_foreground(this->mainThread);
        this->notifyIconPath = filepath;

        if (this->delayNotifyIcon)
        {
            this->SwitchNotifyIcon(true);
        }
    }

    /// <summary>
    /// Initializes the notify icon. Must be called on the UI thread.
    /// </summary>
    void App::SwitchNotifyIcon(bool state)
    {
        if (!this->mainThread.HasThreadAccess()) { throw hresult_wrong_thread(); }

        if (state)
        {
            if (this->notifyIconPath != L"")
            {
                if (!this->notifyIcon)
                {
                    this->notifyIcon = NotifyIcon(this->notifyIconPath, this->appManager.HookEnabled());
                    if (const auto& w{ this->mainWindow.get() }; w) { w.UpdateOpenSettings(this->notifyIcon); }
                    this->openSettingsToken = this->notifyIcon.OnOpenSettings({ this->get_weak(), &App::OnOpenSettings });
                    this->notifyIconSetHookToken = this->notifyIcon.OnSetHookEnabled({ this->get_weak(), &App::OnNotifyIconSetHook });
                    this->appExitToken = this->notifyIcon.OnExitApp({ this->get_weak(), &App::OnAppExit });
                }
            }
            else
            {
                this->delayNotifyIcon = true;
            }
        }
        else
        {
            if (this->notifyIcon)
            {
                this->notifyIcon.OnOpenSettings(this->openSettingsToken);
                this->notifyIcon.OnSetHookEnabled(this->notifyIconSetHookToken);
                this->notifyIcon.OnExitApp(this->appExitToken);
                this->notifyIcon = nullptr;
                if (const auto& w{ this->mainWindow.get() }; w) { w.UpdateOpenSettings(this->notifyIcon); }
            }
        }
    }

    fire_and_forget App::OpenWindow()
    {
        co_await wil::resume_foreground(this->mainThread);
        auto w{ this->mainWindow.get() };
        if (!w)
        {
            w = simbolmudah_ui::MainWindow{
                this->sequenceDefinition, this->appManager, this->notifyIcon };
            this->mainWindow = make_weak(w);
        }
        if (const auto a{ get_self<implementation::AppManager>(this->appManager) }; a->FirstInstall())
        {
            a->FirstInstall(false);
            get_self<implementation::MainWindow>(w)->RootGrid().Loaded([](IInspectable const& src, auto&&) {
                const auto& dialog{ TutorialDialog::GetDialog() };
                dialog.XamlRoot(src.as<Controls::Grid>().XamlRoot());
                dialog.ShowAsync();
            });
        }
        w.Activate();
    }

    fire_and_forget App::OnOpenSettings(NotifyIcon const&, bool)
    {
        using namespace Microsoft::UI::Xaml::Media::Animation;
        using namespace std::chrono_literals;

        co_await wil::resume_foreground(this->mainThread);
        this->OpenWindow();
        co_await 500ms;

        co_await wil::resume_foreground(this->mainThread);
        if (const auto& w{ this->mainWindow.get() }; w)
        {
            get_self<implementation::MainWindow>(w)->NavigateToSettings(EntranceNavigationTransitionInfo());
        }
    }

    fire_and_forget App::OnNotifyIconSetHook(NotifyIcon const&, bool status)
    {
        co_await wil::resume_foreground(this->mainThread);
        this->appManager.HookEnabled(status);
    }

    fire_and_forget App::OnAppExit(NotifyIcon const&, bool)
    {
        co_await wil::resume_foreground(this->mainThread);
        this->Exit();
    }
}

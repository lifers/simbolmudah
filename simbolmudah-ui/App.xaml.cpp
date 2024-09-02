#include "pch.hpp"
#include "App.xaml.h"
#include "MainWindow.xaml.h"
#include <wil/cppwinrt_helpers.h>


namespace winrt::simbolmudah_ui::implementation
{
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
        keyboardThread{ DispatcherQueueController::CreateOnDedicatedThread() },
        appManager{ ApplicationData::Current().LocalSettings() },
        settingsChangedRevoker{ appManager.PropertyChanged(auto_revoke, { this->get_weak(), &App::OnSettingsChanged }) }
    {
        // Initialize the keyboard STA.
        this->keyboardThread.DispatcherQueue().TryEnqueue(
            DispatcherQueuePriority::High,
            []() { init_apartment(apartment_type::single_threaded); }
        );

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
                const auto errorMessage = e.Message();
                ::__debugbreak();
            }
        });
#endif
    }

    fire_and_forget App::InitializeKeyboardHook()
    {
        this->keyboardHook = KeyboardHook{ this->keyboardTranslator };

        co_await wil::resume_foreground(this->mainThread);
        if (this->appManager.UseHookPopup())
        {
            this->popupWindow = simbolmudah_ui::PopupWindow{
                this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition };
        }

        if (this->notifyIcon) { this->notifyIcon.GetHookEnabled(true); }
    }

    /// <summary>
    /// Invoked when the application is launched.
    /// </summary>
    void App::OnLaunched(LaunchActivatedEventArgs const&)
    {
        if (this->appManager.MainWindowOpened())
        {
            this->OpenWindow();
        }

        if (this->appManager.NotifyIconEnabled())
        {
            this->InitializeNotifyIcon();
        }
    }

    /// <summary>
    /// Builds the keyboard translator finite state automaton.
    /// </summary>
    IAsyncAction App::RebuildDefinition()
    {
        if (const auto mainWindowImpl{ get_self<implementation::MainWindow>(this->mainWindow.get()) }; mainWindowImpl)
        {
            mainWindowImpl->SetSequenceDefinition(nullptr);
        }
        co_await resume_background();
        this->sequenceDefinition = nullptr;
        this->keyboardTranslator = nullptr;
        if (this->popupWindow)
        {
            this->popupWindow.Close();
            this->popupWindow = nullptr;
        }
        this->popupWindow = nullptr;
        // TODO: disable the notify icon entry

        const auto keysymdef_path{ StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/keysymdef.h")) };
        const auto composedef_path{ StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/Compose.pre")) };
        this->sequenceDefinition = SequenceDefinition{ (co_await keysymdef_path).Path(), (co_await composedef_path).Path() };
        this->keyboardTranslator = KeyboardTranslator{ this->sequenceDefinition };

        co_await wil::resume_foreground(this->mainThread);
        if (this->appManager.HookEnabled())
        {
            this->keyboardThread.DispatcherQueue().TryEnqueue({ this->get_weak(), &App::InitializeKeyboardHook });
        }

        if (const auto mainWindowImpl{ get_self<implementation::MainWindow>(this->mainWindow.get()) }; mainWindowImpl)
        {
            mainWindowImpl->SetSequenceDefinition(this->sequenceDefinition);
        }
    }

    /// <summary>
    /// Callback for when the settings change.
    /// </summary>
    void App::OnSettingsChanged(IInspectable const&, PropertyChangedEventArgs const&)
    {
        // Update the keyboard hook and popup window.
        // Only do this if the build progress is not running.
        if (this->buildProgress.Status() != AsyncStatus::Started)
        {
            if (this->appManager.HookEnabled() && !this->keyboardHook)
            {
                this->keyboardThread.DispatcherQueue().TryEnqueue({ this->get_weak(), &App::InitializeKeyboardHook });
            }
            else if (!this->appManager.HookEnabled() && this->keyboardHook)
            {
                if (this->notifyIcon) { this->notifyIcon.GetHookEnabled(false); }

                if (this->popupWindow)
                {
                    this->popupWindow.Close();
                    this->popupWindow = nullptr;
                }
                this->keyboardHook = nullptr;
            }

            // Update the popup window, given the hook is enabled.
            if (this->appManager.HookEnabled() && this->appManager.UseHookPopup() && !this->popupWindow)
            {
                this->popupWindow = simbolmudah_ui::PopupWindow{
                    this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition };
            }
            else if (!this->appManager.UseHookPopup() && this->popupWindow)
            {
                this->popupWindow.Close();
                this->popupWindow = nullptr;
            }
        }

        // Update the notify icon and main window.
        if (this->appManager.NotifyIconEnabled() && !this->notifyIcon)
        {
            this->InitializeNotifyIcon();
        }
        else if (!this->appManager.NotifyIconEnabled() && this->notifyIcon)
        {
            this->notifyIcon.OnOpenSettings(this->openSettingsToken);
            this->notifyIcon.OnSetHookEnabled(this->notifyIconSetHookToken);
            this->notifyIcon.OnExitApp(this->appExitToken);
            this->notifyIcon = nullptr;
            if (const auto& w{ this->mainWindow.get() }; w) { w.UpdateOpenSettings(this->notifyIcon); }
        }
    }

    fire_and_forget App::OnNotifyIconPathInitialized(IAsyncOperation<StorageFile> const& op, AsyncStatus)
    {
        const auto filepath{ op.GetResults().Path() };
        co_await wil::resume_foreground(this->mainThread);
        this->notifyIconPath = filepath;

        if (this->delayNotifyIcon)
        {
            this->InitializeNotifyIcon();
        }
    }

    /// <summary>
    /// Initializes the notify icon. Must be called on the UI thread.
    /// </summary>
    void App::InitializeNotifyIcon()
    {
        if (!this->mainThread.HasThreadAccess())
        {
            throw hresult_wrong_thread();
        }
        else if (this->notifyIconPath != L"")
        {
            this->notifyIcon = NotifyIcon(this->notifyIconPath, this->appManager.HookEnabled());
            if (const auto& w{ this->mainWindow.get() }; w) { w.UpdateOpenSettings(this->notifyIcon); }
            this->openSettingsToken = this->notifyIcon.OnOpenSettings({ this->get_weak(), &App::OnOpenSettings });
            this->notifyIconSetHookToken = this->notifyIcon.OnSetHookEnabled({ this->get_weak(), &App::OnNotifyIconSetHook });
            this->appExitToken = this->notifyIcon.OnExitApp({ this->get_weak(), &App::OnAppExit });
        }
        else
        {
            this->delayNotifyIcon = true;
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

#include "pch.hpp"
#include "App.xaml.h"


namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Microsoft::UI::Xaml;
    using namespace Data;
    using namespace Windows;
    using namespace Storage;

    /// <summary>
    /// Initializes the singleton application object.  This is the first line of authored code
    /// executed, and as such is the logical equivalent of main() or WinMain().
    /// </summary>
    App::App() :
        main_thread{ apartment_context() },
        appManager{ ApplicationData::Current().LocalSettings() },
        keyboardTranslator{ sequenceDefinition },
        settingsChangedRevoker{ appManager.PropertyChanged(auto_revoke, { this->get_weak(), &App::OnSettingsChanged }) }
    {
        // Xaml objects should not call InitializeComponent during construction.
        // See https://github.com/microsoft/cppwinrt/tree/master/nuget#initializecomponent

        // Do not close the application when the last window is closed.
        this->DispatcherShutdownMode(DispatcherShutdownMode::OnExplicitShutdown);

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

    /// <summary>
    /// Invoked when the application is launched.
    /// </summary>
    void App::OnLaunched(LaunchActivatedEventArgs const&)
    {
        // Build the keyboard translator finite state automaton.
        this->BuildDefinition();

        if (this->appManager.NotifyIconEnabled())
        {
            this->notifyIcon = NotifyIcon(this->appManager.HookEnabled());
            this->openSettingsToken = this->notifyIcon.OnOpenSettings({ this->get_weak(), &App::OnOpenSettings });
            this->notifyIconSetHookToken = this->notifyIcon.OnSetHookEnabled({ this->get_weak(), &App::OnNotifyIconSetHook });
            this->appExitToken = this->notifyIcon.OnExitApp({ this->get_weak(), &App::OnAppExit });
        }

        if (this->appManager.HookEnabled())
        {
            this->keyboardHook = KeyboardHook{ this->keyboardTranslator };

            if (this->appManager.UseHookPopup())
            {
                this->popupWindow = simbolmudah_ui::PopupWindow{
                    this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition };
            }
        }

        if (this->appManager.MainWindowOpened())
        {
            this->OpenWindow();
        }
    }

    /// <summary>
    /// Builds the keyboard translator finite state automaton.
    /// </summary>
    fire_and_forget App::BuildDefinition() const
    {
        using namespace Foundation;

        const auto keysymdef_path{ StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/keysymdef.txt")) };
        const auto composedef_path{ StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/Compose.pre")) };
        this->sequenceDefinition.Build((co_await keysymdef_path).Path(), (co_await composedef_path).Path());
    }

    /// <summary>
    /// Callback for when the settings change.
    /// </summary>
    void App::OnSettingsChanged(IInspectable const&, PropertyChangedEventArgs const&)
    {
        // Update the keyboard hook and popup window.
        if (this->appManager.HookEnabled() && !this->keyboardHook)
        {
            this->keyboardHook = KeyboardHook{ this->keyboardTranslator };

            if (this->appManager.UseHookPopup())
            {
                this->popupWindow = simbolmudah_ui::PopupWindow{
                    this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition };
            }

            if (this->notifyIcon) { this->notifyIcon.GetHookEnabled(true); }
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

        // Update the notify icon and main window.
        if (this->appManager.NotifyIconEnabled() && !this->notifyIcon)
        {
            this->notifyIcon = NotifyIcon(this->appManager.HookEnabled());
            if (const auto& w{ this->mainWindow.get() }; w) { w.UpdateOpenSettings(this->notifyIcon); }
            this->openSettingsToken = this->notifyIcon.OnOpenSettings({ this->get_weak(), &App::OnOpenSettings });
            this->notifyIconSetHookToken = this->notifyIcon.OnSetHookEnabled({ this->get_weak(), &App::OnNotifyIconSetHook });
            this->appExitToken = this->notifyIcon.OnExitApp({ this->get_weak(), &App::OnAppExit });
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

    fire_and_forget App::OpenWindow()
    {
        co_await this->main_thread;
        auto w{ this->mainWindow.get() };
        if (!w)
        {
            w = simbolmudah_ui::MainWindow{
                this->sequenceDefinition, this->appManager, this->notifyIcon, 0 };
            this->mainWindow = make_weak(w);
        }
        w.Activate();
    }

    fire_and_forget App::OnOpenSettings(NotifyIcon const&, bool)
    {
        using namespace Microsoft::UI::Xaml::Media::Animation;
        using namespace std::chrono_literals;

        co_await this->main_thread;
        this->OpenWindow();
        co_await 500ms;

        co_await this->main_thread;
        if (const auto& w{ this->mainWindow.get() }; w)
        {
            w.NavigateToSettings(EntranceNavigationTransitionInfo());
        }
    }

    fire_and_forget App::OnNotifyIconSetHook(NotifyIcon const&, bool status)
    {
        co_await this->main_thread;
        this->appManager.HookEnabled(status);
    }

    fire_and_forget App::OnAppExit(NotifyIcon const&, bool)
    {
        co_await this->main_thread;
        this->Exit();
    }
}

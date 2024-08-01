#include "pch.hpp"
#include "App.xaml.h"

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

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
        Application::Current().DispatcherShutdownMode(DispatcherShutdownMode::OnExplicitShutdown);

#if defined _DEBUG && !defined DISABLE_XAML_GENERATED_BREAK_ON_UNHANDLED_EXCEPTION
        UnhandledException([](IInspectable const&, UnhandledExceptionEventArgs const& e)
        {
            if (IsDebuggerPresent())
            {
                auto errorMessage = e.Message();
                __debugbreak();
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
        }

        if (this->appManager.HookEnabled())
        {
            this->keyboardHook = KeyboardHook{ this->keyboardTranslator };
            this->popupWindow = simbolmudah_ui::PopupWindow{
                this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition };
        }

        if (this->appManager.MainWindowOpened())
        {
            this->mainWindow = simbolmudah_ui::MainWindow{
                this->sequenceDefinition, this->appManager, this->notifyIcon, 0 };
            this->mainWindow.Closed({ this->get_weak(), &App::WindowClosed });
            this->mainWindow.Activate();
        }
    }

    /// <summary>
    /// Callback for when the main window is closed.
    /// </summary>
    void App::WindowClosed(IInspectable const&, WindowEventArgs const&) { this->mainWindow = nullptr; }

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
            this->popupWindow = simbolmudah_ui::PopupWindow{
                this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition };

            if (this->notifyIcon) { this->notifyIcon.GetHookEnabled(true); }
        }
        else if (!this->appManager.HookEnabled() && this->keyboardHook)
        {
            if (this->notifyIcon) { this->notifyIcon.GetHookEnabled(false); }
            this->popupWindow = nullptr;
            this->keyboardHook = nullptr;
        }

        // Update the notify icon and main window.
        if (this->appManager.NotifyIconEnabled() && !this->notifyIcon)
        {
            this->notifyIcon = NotifyIcon(this->appManager.HookEnabled());
            this->mainWindow.UpdateOpenSettings(this->notifyIcon);
            this->openSettingsToken = this->notifyIcon.OnOpenSettings({ this->get_weak(), &App::OnOpenSettings });
            this->notifyIconSetHookToken = this->notifyIcon.OnSetHookEnabled({ this->get_weak(), &App::OnNotifyIconSetHook });
        }
        else if (!this->appManager.NotifyIconEnabled() && this->notifyIcon)
        {
            this->notifyIcon.OnOpenSettings(this->openSettingsToken);
            this->notifyIcon.OnSetHookEnabled(this->notifyIconSetHookToken);
            this->notifyIcon = nullptr;
            this->mainWindow.UpdateOpenSettings(this->notifyIcon);
        }
    }

    fire_and_forget App::OnOpenSettings(NotifyIcon const&, bool)
    {
        using namespace Microsoft::UI::Xaml::Media::Animation;
        using namespace std::chrono_literals;

        co_await this->main_thread;
        if (!this->mainWindow)
        {
            this->mainWindow = simbolmudah_ui::MainWindow{
                this->sequenceDefinition, this->appManager, this->notifyIcon, 1 };
            this->mainWindow.Closed({ this->get_weak(), &App::WindowClosed });
            this->mainWindow.Activate();
            co_await 500ms;
            co_await this->main_thread;
            this->mainWindow.NavigateToSettings(EntranceNavigationTransitionInfo());
        }
    }

    fire_and_forget App::OnNotifyIconSetHook(NotifyIcon const&, bool status)
    {
        co_await this->main_thread;
        this->appManager.HookEnabled(status);
    }
}

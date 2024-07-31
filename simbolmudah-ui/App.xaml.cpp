module;
#include "pch.hpp"
#include <winrt/simbolmudah_ui.h>
module App;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Microsoft::UI::Xaml;
    using namespace Controls;
    using namespace Data;
    using namespace Media::Animation;
    using namespace Windows;
    using namespace Foundation;
    using namespace Storage;
    using namespace std::chrono_literals;

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
            this->notifyIcon = LibSimbolMudah::NotifyIcon();
            this->openSettingsToken = this->notifyIcon.OnOpenSettings({ this->get_weak(), &App::OnOpenSettings });
        }

        if (this->appManager.HookEnabled())
        {
            this->keyboardHook = LibSimbolMudah::KeyboardHook{ this->keyboardTranslator };
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
            this->keyboardHook = LibSimbolMudah::KeyboardHook{ this->keyboardTranslator };
            this->popupWindow = simbolmudah_ui::PopupWindow{
                this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition };
        }
        else if (!this->appManager.HookEnabled() && this->keyboardHook)
        {
            this->popupWindow = nullptr;
            this->keyboardHook = nullptr;
        }

        // Update the notify icon and main window.
        if (this->appManager.NotifyIconEnabled() && !this->notifyIcon)
        {
            this->notifyIcon = LibSimbolMudah::NotifyIcon();
            this->mainWindow.UpdateOpenSettings(this->notifyIcon);
            this->openSettingsToken = this->notifyIcon.OnOpenSettings({ this->get_weak(), &App::OnOpenSettings });
        }
        else if (!this->appManager.NotifyIconEnabled() && this->notifyIcon)
        {
            this->notifyIcon.OnOpenSettings(this->openSettingsToken);
            this->notifyIcon = nullptr;
            this->mainWindow.UpdateOpenSettings(this->notifyIcon);
        }
    }

    fire_and_forget App::OnOpenSettings(LibSimbolMudah::NotifyIcon const&, bool)
    {
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
}

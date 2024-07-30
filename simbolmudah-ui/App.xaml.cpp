#include "pch.hpp"
#include "App.xaml.h"
#include <wil/resource.h>

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Microsoft::UI::Xaml;
    using namespace Controls;
    using namespace Windows;
    using namespace Foundation;
    using namespace Storage;

    /// <summary>
    /// Initializes the singleton application object.  This is the first line of authored code
    /// executed, and as such is the logical equivalent of main() or WinMain().
    /// </summary>
    App::App() : main_thread{ apartment_context() }, keyboardTranslator{ sequenceDefinition }
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
        this->BuildDefinition();
        this->InitializeSettings();
        
        this->notifyIcon = LibSimbolMudah::NotifyIcon();

        this->window = simbolmudah_ui::MainWindow(0ui8);
        this->window.Closed([this](auto&&, auto&&) { this->window = nullptr; });
        this->window.ExtendsContentIntoTitleBar(true);
        this->window.Activate();

        this->onSettingsOpenedToken = this->notifyIcon.OnOpenSettings({ this->get_weak(), &App::OnOpenSettings });
    }

    /// <summary>
    /// Invoked when the keyboard hook switch is toggled.
    /// </summary>
    void App::OnHookToggle(bool isOn)
    {
        if (isOn)
        {
            this->keyboardHook = LibSimbolMudah::KeyboardHook(this->keyboardTranslator);
            this->popup = simbolmudah_ui::PopupWindow(
                this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition);
        }
        else
        {
            this->popup = nullptr;
            this->keyboardHook = nullptr;
        }
    }

    /// <summary>
    /// Builds the keyboard translator finite state automaton.
    /// </summary>
    /// <returns></returns>
    fire_and_forget App::BuildDefinition() const
    {
        const auto keysymdef_path{ StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/keysymdef.txt")) };
        const auto composedef_path{ StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/Compose.pre")) };
        this->sequenceDefinition.Build((co_await keysymdef_path).Path(), (co_await composedef_path).Path());
    }

    /// <summary>
    /// Initializes the application settings.
    /// </summary>
    void App::InitializeSettings()
    {
        if (const auto& localSettings{ ApplicationData::Current().LocalSettings().Values() }; !localSettings.HasKey(L"keyboardHookEnabled"))
        {
            localSettings.Insert(L"keyboardHookEnabled", box_value(false));
        }
        else if (unbox_value<bool>(localSettings.Lookup(L"keyboardHookEnabled")))
        {
            this->keyboardHook = LibSimbolMudah::KeyboardHook(this->keyboardTranslator);
            this->popup = simbolmudah_ui::PopupWindow(
                this->keyboardTranslator, this->keyboardHook, this->sequenceDefinition);
            this->hookState = true;
        }
    }

    /// <summary>
    /// Invoked when the "Open setting" button in the notification menu is clicked.
    /// </summary>
    fire_and_forget App::OnOpenSettings(NotifyIcon const&, bool)
    {
        co_await this->main_thread;
        if (!this->window)
        {
            this->window = simbolmudah_ui::MainWindow(1);
            this->window.Closed([this](auto&&, auto&&) { this->window = nullptr; });
            this->window.ExtendsContentIntoTitleBar(true);
            this->window.Activate();
        }
        else
        {
            this->window.OpenSettings();
            this->window.Activate();
        }
    }
}

namespace
{
    using namespace wil;
    using namespace winrt::Microsoft::Windows::AppLifecycle;

    winrt::fire_and_forget Redirect(AppInstance keyInstance, AppActivationArguments args, unique_event& redirectHandle)
    {
        const auto ensure_signaled{ SetEvent_scope_exit(redirectHandle.get()) };
        co_await keyInstance.RedirectActivationToAsync(args);
    }
}

int WINAPI wWinMain(_In_ HINSTANCE, _In_opt_ HINSTANCE, _In_ LPWSTR, _In_ int)
{
    using namespace winrt;
    using namespace winrt::Microsoft::UI::Xaml;
    using namespace winrt::Microsoft::Windows::AppLifecycle;
    using namespace Windows::Foundation;

    init_apartment(apartment_type::single_threaded);

    if (const auto keyInstance{ AppInstance::FindOrRegisterForKey(L"simbolmudah") };
        keyInstance.IsCurrent())
    {
        Application::Start([](auto&&) { make<simbolmudah_ui::implementation::App>(); });
    }
    else
    {
        wil::unique_event redirectHandle;
        redirectHandle.create();
        Redirect(keyInstance, AppInstance::GetCurrent().GetActivatedEventArgs(), redirectHandle);
        DWORD handleIndex;
        check_hresult(CoWaitForMultipleObjects(CWMO_DEFAULT, INFINITE, 1, redirectHandle.addressof(), &handleIndex));
    }

    return 0;
}
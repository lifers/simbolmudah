#include "pch.h"
#include "App.xaml.h"

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
    using namespace Microsoft::UI::Xaml;
    using namespace Controls;
    using namespace Windows;
    using namespace Foundation;
    using namespace Storage;

    /// <summary>
    /// Initializes the singleton application object.  This is the first line of authored code
    /// executed, and as such is the logical equivalent of main() or WinMain().
    /// </summary>
    App::App() : main_thread{ apartment_context() }
    {
        // Xaml objects should not call InitializeComponent during construction.
        // See https://github.com/microsoft/cppwinrt/tree/master/nuget#initializecomponent

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
    /// Destroy the app
    /// <summary>
    App::~App()
	{
        if (this->keyboardHook)
		{
			this->keyboardHook->Deactivate();
		}
	}

    /// <summary>
    /// Invoked when the application is launched.
    /// </summary>
    void App::OnLaunched(LaunchActivatedEventArgs const&)
    {
        this->BuildTranslator();
        this->InitializeSettings();
        
        this->window = simbolmudah_ui::MainWindow();
        this->window.ExtendsContentIntoTitleBar(true);
        this->window.Activate();
    }

    /// <summary>
    /// Invoked when the keyboard hook switch is toggled.
    /// </summary>
    void App::OnHookToggle(bool isOn)
    {
        if (isOn)
        {
            this->keyboardHook.emplace(this->keyboardTranslator);
        }
		else
		{
			this->keyboardHook->Deactivate();
			this->keyboardHook.reset();
		}
    }

    /// <summary>
    /// Builds the keyboard translator finite state automaton.
    /// </summary>
    /// <returns></returns>
    fire_and_forget App::BuildTranslator() const
	{
		const auto keysymdef_path = StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/keysymdef.h"));
		const auto composedef_path = StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/Compose.pre"));
		this->keyboardTranslator.BuildTranslator((co_await keysymdef_path).Path(), (co_await composedef_path).Path());
	}

    /// <summary>
	/// Initializes the application settings.
	/// </summary>
	void App::InitializeSettings()
	{
		if (const auto localSettings = ApplicationData::Current().LocalSettings().Values(); !localSettings.HasKey(L"keyboardHookEnabled"))
		{
			localSettings.Insert(L"keyboardHookEnabled", box_value(false));
		}
        else if (unbox_value<bool>(localSettings.Lookup(L"keyboardHookEnabled")))
		{
			this->keyboardHook.emplace(this->keyboardTranslator);
            this->hookState = true;
		}
	}
}

#include "pch.h"
#include "HomePage.xaml.h"
#if __has_include("HomePage.g.cpp")
#include "HomePage.g.cpp"
#endif

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
	using namespace Windows;
	using namespace Foundation;
	using namespace Storage;
	using namespace Microsoft::UI::Xaml;

	HomePage::HomePage() : main_thread(apartment_context())
	{
		this->showResultsToken = this->keyboardTranslator.OnTranslated(
			TypedEventHandler<KeyboardTranslator, hstring>::TypedEventHandler(this, &HomePage::ShowResult)
		);
		this->BuildTranslator();
	}

	HomePage::~HomePage()
	{
		this->keyboardTranslator.OnTranslated(this->showResultsToken);
	}
	
	void HomePage::ListenKeyUpdate(const IInspectable&, const RoutedEventArgs&)
	{	
		if (this->listenKeySwitch().IsOn())
		{
			this->keyboardHook.emplace(this->keyboardTranslator);
			this->infoUpdaterToken = this->keyboardHook->DebugKeyEvent(
				TypedEventHandler<KeyboardHook, hstring>::TypedEventHandler(this, &HomePage::InfoUpdater)
			);
			this->stateUpdaterToken = this->keyboardHook->DebugStateChanged(
				TypedEventHandler<KeyboardHook, hstring>::TypedEventHandler(this, &HomePage::StateUpdater)
			);
		}
		else
		{
			this->keyboardHook->DebugKeyEvent(this->infoUpdaterToken);
			this->keyboardHook->DebugStateChanged(this->stateUpdaterToken);
			this->keyboardHook->Deactivate();
			this->keyboardHook.reset(); 
		}
	}

	fire_and_forget HomePage::BuildTranslator() const
	{
		co_await resume_background();
		const auto keysymdef_path = StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/keysymdef.h"));
		const auto composedef_path = StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/Compose.pre"));
		this->keyboardTranslator.BuildTranslator(keysymdef_path.get().Path(), composedef_path.get().Path());
	}

	fire_and_forget HomePage::InfoUpdater(const KeyboardHook&, const hstring& message)
	{
		const hstring result{ message };
		co_await this->main_thread;
		this->infoBar().Message(result);
		this->infoBar().IsOpen(true);
	}

	fire_and_forget HomePage::StateUpdater(const KeyboardHook&, const hstring& message)
	{
		const hstring result{ message };
		co_await this->main_thread;
		this->stateBar().Message(result);
		this->stateBar().IsOpen(true);
	}

	fire_and_forget HomePage::ShowResult(const KeyboardTranslator&, const hstring& message)
	{
		const hstring result{ message };
		co_await this->main_thread;
		this->resultBar().Message(result);
		this->resultBar().IsOpen(true);
	}
}

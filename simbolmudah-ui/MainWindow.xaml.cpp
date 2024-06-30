#include "pch.h"
#include "MainWindow.xaml.h"
#if __has_include("MainWindow.g.cpp")
#include "MainWindow.g.cpp"
#endif

using namespace winrt;
using namespace LibSimbolMudah;
using namespace Microsoft::UI::Xaml;
using namespace Windows::Foundation;
using namespace Windows::Storage;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
	MainWindow::MainWindow() : main_thread(apartment_context())
	{
		this->showResultsToken = this->keyboardTranslator.OnTranslated(
			TypedEventHandler<KeyboardTranslator, hstring>::TypedEventHandler(this, &MainWindow::ShowResult)
		);
		this->BuildTranslator();
	}

	MainWindow::~MainWindow()
	{
		this->keyboardTranslator.OnTranslated(this->showResultsToken);
	}
	
	void MainWindow::ListenKeyUpdate(const IInspectable&, const RoutedEventArgs&)
	{	
		if (this->listenKeySwitch().IsOn())
		{
			this->keyboardHook.emplace(this->keyboardTranslator);
			this->infoUpdaterToken = this->keyboardHook->DebugKeyEvent(
				TypedEventHandler<KeyboardHook, hstring>::TypedEventHandler(this, &MainWindow::InfoUpdater)
			);
			this->stateUpdaterToken = this->keyboardHook->DebugStateChanged(
				TypedEventHandler<KeyboardHook, hstring>::TypedEventHandler(this, &MainWindow::StateUpdater)
			);
		}
		else
		{
			this->keyboardHook->DebugKeyEvent(this->infoUpdaterToken);
			this->keyboardHook->DebugStateChanged(this->stateUpdaterToken);
			this->keyboardHook.reset(); 
		}
	}

	fire_and_forget MainWindow::BuildTranslator() const
	{
		co_await resume_background();
		const auto keysymdef_path = StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/keysymdef.h"));
		const auto composedef_path = StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/Compose.pre"));
		this->keyboardTranslator.BuildTranslator(keysymdef_path.get().Path(), composedef_path.get().Path());
	}

	fire_and_forget MainWindow::InfoUpdater(const KeyboardHook&, const hstring& message)
	{
		const hstring result{ message };
		co_await this->main_thread;
		this->infoBar().Message(result);
		this->infoBar().IsOpen(true);
	}

	fire_and_forget MainWindow::StateUpdater(const KeyboardHook&, const hstring& message)
	{
		const hstring result{ message };
		co_await this->main_thread;
		this->stateBar().Message(result);
		this->stateBar().IsOpen(true);
	}

	fire_and_forget MainWindow::ShowResult(const KeyboardTranslator&, const hstring& message)
	{
		const hstring result{ message };
		co_await this->main_thread;
		this->resultBar().Message(result);
		this->resultBar().IsOpen(true);
	}
}

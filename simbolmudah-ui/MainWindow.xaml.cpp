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
			const delegate<fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)> infoUpdater{
				this, &MainWindow::InfoUpdater
			};
			const delegate<fire_and_forget(std::wstring)> stateUpdater{
				this, &MainWindow::StateUpdater
			};

			this->keyboardHook.emplace(
				infoUpdater,
				stateUpdater,
				this->keyboardTranslator
			);
		}
		else
		{
			this->keyboardHook.reset(); 
			//this->keyboardTranslator.Flush();
		}
	}

	winrt::fire_and_forget MainWindow::BuildTranslator() const
	{
		co_await resume_background();
		const auto keysymdef_path = StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/keysymdef.h"));
		const auto composedef_path = StorageFile::GetFileFromApplicationUriAsync(Uri(L"ms-appx:///Assets/Resources/Compose.pre"));
		this->keyboardTranslator.BuildTranslator(keysymdef_path.get().Path(), composedef_path.get().Path());
	}

	fire_and_forget MainWindow::InfoUpdater(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage)
	{
		const auto keyEventCopy{ keyEvent };
		const auto windowMessageCopy{ windowMessage };
		co_await this->main_thread;
		this->infoBar().Message(std::format(
			L"vkCode: {}\nscanCode: {}\nflags: {}\ntime: {}\ndwExtraInfo: {}\nwParam: {}.",
			keyEventCopy.vkCode, keyEventCopy.scanCode, keyEventCopy.flags, keyEventCopy.time, keyEventCopy.dwExtraInfo, windowMessageCopy
		));
		this->infoBar().IsOpen(true);
	}

	fire_and_forget MainWindow::StateUpdater(std::wstring message)
	{
		const hstring result{ message };
		co_await this->main_thread;
		this->stateBar().Message(result);
		this->stateBar().IsOpen(true);
	}

	fire_and_forget MainWindow::ShowResult(KeyboardTranslator const&, hstring const& message)
	{
		const hstring result{ message };
		co_await this->main_thread;
		this->resultBar().Message(result);
		this->resultBar().IsOpen(true);
	}
}

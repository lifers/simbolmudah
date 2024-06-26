#include "pch.h"
#include "MainWindow.xaml.h"
#if __has_include("MainWindow.g.cpp")
#include "MainWindow.g.cpp"
#endif

using namespace winrt;
using namespace LibSimbolMudah;
using namespace Microsoft::UI::Xaml;
using namespace Windows::Foundation;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
	MainWindow::MainWindow() : main_thread(apartment_context())
	{
		wchar_t buffer[1024];
		GetCurrentDirectoryW(1024, buffer);
		MessageBoxW(nullptr, buffer, L"Current Directory", MB_OK);

		this->showResultsToken = this->keyboardTranslator.OnTranslated(
			TypedEventHandler<KeyboardTranslator, hstring>::TypedEventHandler(this, &MainWindow::ShowResult)
		);
		this->keyboardTranslator.BuildTranslator();
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

	fire_and_forget MainWindow::InfoUpdater(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage)
	{
		co_await this->main_thread;
		this->infoBar().Message(std::format(
			L"vkCode: {}\nscanCode: {}\nflags: {}\ntime: {}\ndwExtraInfo: {}\nwParam: {}.",
			keyEvent.vkCode, keyEvent.scanCode, keyEvent.flags, keyEvent.time, keyEvent.dwExtraInfo, windowMessage
		));
		this->infoBar().IsOpen(true);
	}

	fire_and_forget MainWindow::StateUpdater(std::wstring message)
	{
		co_await this->main_thread;
		this->stateBar().Message(message);
		this->stateBar().IsOpen(true);
	}

	fire_and_forget MainWindow::ShowResult(KeyboardTranslator const&, hstring const& message)
	{
		co_await this->main_thread;
		this->resultBar().Message(message);
		this->resultBar().IsOpen(true);
	}
}

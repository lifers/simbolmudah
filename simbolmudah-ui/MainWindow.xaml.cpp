#include "pch.h"
#include "MainWindow.xaml.h"
#if __has_include("MainWindow.g.cpp")
#include "MainWindow.g.cpp"
#endif

using namespace winrt;
using namespace Microsoft::UI::Xaml;
using namespace Windows::Foundation;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
	void MainWindow::ListenKeyUpdate(const IInspectable&, const RoutedEventArgs&)
	{	
		if (this->listenKeySwitch().IsOn())
		{
			this->keyboardHook.emplace(
				delegate<fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)>{ this, &MainWindow::InfoUpdater },
				delegate<fire_and_forget(std::wstring)>{ this, &MainWindow::StateUpdater }
			);
		}
		else
		{
			this->keyboardHook.reset();
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
}

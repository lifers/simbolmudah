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
		if (listenKeySwitch().IsOn())
		{
			keyboardHook.emplace(
				delegate<LowLevelKeyboardEvent>{ this, &MainWindow::UpdateInfoBar },
				delegate<std::wstring>{ this, &MainWindow::UpdateStateBar }
			);
		}
		else
		{
			keyboardHook.reset();
		}
	}

	fire_and_forget MainWindow::UpdateInfoBar(LowLevelKeyboardEvent keyEvent)
	{
		co_await main_thread;
		infoBar().Message(std::format(
			L"vkCode: {}\nscanCode: {}\nflags: {}\ntime: {}\ndwExtraInfo: {}\nwParam: {}.",
			keyEvent.vkCode, keyEvent.scanCode, keyEvent.flags, keyEvent.time, keyEvent.dwExtraInfo, keyEvent.windowMessage
		));
		infoBar().IsOpen(true);
	}

	fire_and_forget MainWindow::UpdateStateBar(std::wstring message)
	{
		co_await main_thread;
		stateBar().Message(message);
		stateBar().IsOpen(true);
	}
}

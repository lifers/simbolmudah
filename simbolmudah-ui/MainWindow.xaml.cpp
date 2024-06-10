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
	MainWindow::MainWindow() :
		main_thread{ apartment_context() },
		infoUpdater{
			[this](KBDLLHOOKSTRUCT keyEvent, WPARAM wParam) -> fire_and_forget
			{
				co_await this->main_thread;
				infoBar().Message(std::format(
					L"vkCode: {}\nscanCode: {}\nflags: {}\ntime: {}\ndwExtraInfo: {}\nwParam: {}.",
					keyEvent.vkCode, keyEvent.scanCode, keyEvent.flags, keyEvent.time, keyEvent.dwExtraInfo, wParam
				));
				infoBar().IsOpen(true);
			}
		},
		stateUpdater{
			[this](std::wstring message) -> fire_and_forget
			{
				co_await this->main_thread;
				stateBar().Message(message);
				stateBar().IsOpen(true);
			}
		}
	{}

	void MainWindow::ListenKeyUpdate(const IInspectable&, const RoutedEventArgs&)
	{	
		if (listenKeySwitch().IsOn())
		{
			keyboardHook.emplace(infoUpdater, stateUpdater);
		}
		else
		{
			keyboardHook.reset();
		}
	}
}

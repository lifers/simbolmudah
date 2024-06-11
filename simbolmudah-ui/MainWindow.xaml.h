#pragma once

#include "MainWindow.g.h"
import std;
import KeyboardHook;

namespace winrt::simbolmudah_ui::implementation
{
	struct MainWindow : MainWindowT<MainWindow>
	{
		MainWindow() : main_thread(apartment_context()) {}
		void ListenKeyUpdate(const IInspectable& sender, const Microsoft::UI::Xaml::RoutedEventArgs& args);

	private:
		winrt::fire_and_forget InfoUpdater(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage);
		winrt::fire_and_forget StateUpdater(std::wstring message);
		std::optional<KeyboardHook> keyboardHook;
		const apartment_context main_thread;
	};
}

namespace winrt::simbolmudah_ui::factory_implementation
{
	struct MainWindow : MainWindowT<MainWindow, implementation::MainWindow>
	{
	};
}

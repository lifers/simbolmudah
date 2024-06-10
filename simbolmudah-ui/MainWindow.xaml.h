#pragma once

#include "MainWindow.g.h"
import std;
import KeyboardHook;

namespace winrt::simbolmudah_ui::implementation
{
	struct MainWindow : MainWindowT<MainWindow>
	{
		MainWindow();
		void ListenKeyUpdate(const IInspectable& sender, const Microsoft::UI::Xaml::RoutedEventArgs& args);

	private:
		const std::function<winrt::fire_and_forget(KBDLLHOOKSTRUCT, WPARAM)> infoUpdater;
		const std::function<winrt::fire_and_forget(std::wstring)> stateUpdater;
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

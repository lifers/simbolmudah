#pragma once

#include "MainWindow.g.h"
import std.core;
import KeyboardHook;
import KeyboardTranslator;

namespace winrt::simbolmudah_ui::implementation
{
	struct MainWindow : MainWindowT<MainWindow>
	{
		MainWindow() : main_thread(apartment_context()) {}
		MainWindow(const MainWindow&) = delete;
		MainWindow& operator=(const MainWindow&) = delete;
		void ListenKeyUpdate(const IInspectable& sender, const Microsoft::UI::Xaml::RoutedEventArgs& args);

	private:
		winrt::fire_and_forget InfoUpdater(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage);
		winrt::fire_and_forget StateUpdater(std::wstring message);

		KeyboardTranslator keyboardTranslator;
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

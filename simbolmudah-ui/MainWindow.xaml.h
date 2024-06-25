#pragma once

#include "MainWindow.g.h"
#include <winrt/LibSimbolMudah.h>
import std.core;
import KeyboardHook;
//import KeyboardTranslator;

namespace winrt::simbolmudah_ui::implementation
{
	struct MainWindow : MainWindowT<MainWindow>
	{
		MainWindow();
		~MainWindow();
		MainWindow(const MainWindow&) = delete;
		MainWindow& operator=(const MainWindow&) = delete;
		void ListenKeyUpdate(const IInspectable& sender, const Microsoft::UI::Xaml::RoutedEventArgs& args);

	private:
		winrt::fire_and_forget InfoUpdater(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage);
		winrt::fire_and_forget StateUpdater(std::wstring message);
		winrt::fire_and_forget ShowResult(winrt::LibSimbolMudah::KeyboardTranslator const&, hstring const& message);

		winrt::LibSimbolMudah::KeyboardTranslator keyboardTranslator;
		std::optional<KeyboardHook> keyboardHook;
		const apartment_context main_thread;
		winrt::event_token showResultsToken;
	};
}

namespace winrt::simbolmudah_ui::factory_implementation
{
	struct MainWindow : MainWindowT<MainWindow, implementation::MainWindow>
	{
	};
}

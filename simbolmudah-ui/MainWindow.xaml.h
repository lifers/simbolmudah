#pragma once

#include "MainWindow.g.h"
#include <winrt/LibSimbolMudah.h>
#include <optional>

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
		fire_and_forget InfoUpdater(const LibSimbolMudah::KeyboardHook&, const hstring& message);
		fire_and_forget StateUpdater(const LibSimbolMudah::KeyboardHook&, const hstring& message);
		fire_and_forget ShowResult(const LibSimbolMudah::KeyboardTranslator&, const hstring& message);
		fire_and_forget BuildTranslator() const;

		const LibSimbolMudah::KeyboardTranslator keyboardTranslator;
		std::optional<LibSimbolMudah::KeyboardHook> keyboardHook;
		const apartment_context main_thread;
		event_token showResultsToken;
		event_token infoUpdaterToken;
		event_token stateUpdaterToken;
	};
}

namespace winrt::simbolmudah_ui::factory_implementation
{
	struct MainWindow : MainWindowT<MainWindow, implementation::MainWindow>
	{
	};
}

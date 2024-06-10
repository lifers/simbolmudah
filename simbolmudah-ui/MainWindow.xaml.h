#pragma once

#include "MainWindow.g.h"
import std;
import Core;
import KeyboardHook;

namespace winrt::simbolmudah_ui::implementation
{
	struct MainWindow : MainWindowT<MainWindow>
	{
		MainWindow() : main_thread(apartment_context())
		{
			// Xaml objects should not call InitializeComponent during construction.
			// See https://github.com/microsoft/cppwinrt/tree/master/nuget#initializecomponent
		}

		void ListenKeyUpdate(const IInspectable& sender, const Microsoft::UI::Xaml::RoutedEventArgs& args);

	private:
		fire_and_forget UpdateInfoBar(LowLevelKeyboardEvent keyEvent);
		fire_and_forget UpdateStateBar(std::wstring message);
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

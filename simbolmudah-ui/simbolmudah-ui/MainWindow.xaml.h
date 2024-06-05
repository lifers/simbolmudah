#pragma once

#include "MainWindow.g.h"
import KeyboardHook;

namespace winrt::simbolmudah_ui::implementation
{
	struct MainWindow : MainWindowT<MainWindow>
	{
		MainWindow()
		{
			// Xaml objects should not call InitializeComponent during construction.
			// See https://github.com/microsoft/cppwinrt/tree/master/nuget#initializecomponent
			this->keyboardHook = KeyboardHook([this](DWORD vkCode) { this->UpdateInfoBar(vkCode); });
		}

		void ListenKeyUpdate(const IInspectable& sender, const Microsoft::UI::Xaml::RoutedEventArgs& args);

	private:
		fire_and_forget UpdateInfoBar(DWORD vkCode);
		KeyboardHook keyboardHook;
		apartment_context main_thread;
		Windows::Foundation::IAsyncAction listenerHandle;
	};
}

namespace winrt::simbolmudah_ui::factory_implementation
{
	struct MainWindow : MainWindowT<MainWindow, implementation::MainWindow>
	{
	};
}

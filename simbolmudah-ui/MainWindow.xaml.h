#pragma once

#include "MainWindow.g.h"

namespace winrt::simbolmudah_ui::implementation
{
	struct MainWindow : MainWindowT<MainWindow>
	{
		void ContentFrame_Navigated(IInspectable const&, Microsoft::UI::Xaml::Navigation::NavigationEventArgs const&);
		void ContentFrame_NavigationFailed(IInspectable const&, Microsoft::UI::Xaml::Navigation::NavigationFailedEventArgs const& e);
		void NavigationViewControl_Loaded(IInspectable const&, Microsoft::UI::Xaml::RoutedEventArgs const&);
		void NavigationViewControl_ItemInvoked(
			Microsoft::UI::Xaml::Controls::NavigationView const&,
			Microsoft::UI::Xaml::Controls::NavigationViewItemInvokedEventArgs const& args);
		void NavigationViewControl_BackRequested(
			Microsoft::UI::Xaml::Controls::NavigationView const&,
			Microsoft::UI::Xaml::Controls::NavigationViewBackRequestedEventArgs const&);
		void Window_SizeChanged(IInspectable const&, Microsoft::UI::Xaml::WindowSizeChangedEventArgs const& args);

	private:
		void NavigateInternal(
			hstring const& navPageType,
			Microsoft::UI::Xaml::Media::Animation::NavigationTransitionInfo const& transitionInfo);
	};
}

namespace winrt::simbolmudah_ui::factory_implementation
{
	struct MainWindow : MainWindowT<MainWindow, implementation::MainWindow>
	{
	};
}

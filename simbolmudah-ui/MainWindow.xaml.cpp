#include "pch.hpp"
#include "App.xaml.h"
#include "MainWindow.xaml.h"
#if __has_include("MainWindow.g.cpp")
#include "MainWindow.g.cpp"
#endif

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
	using namespace Microsoft::UI::Xaml;
	using namespace Navigation;
	using namespace Controls;
	using namespace Media::Animation;
	using namespace Windows::UI::Xaml::Interop;

	void MainWindow::ContentFrame_Navigated(IInspectable const&, NavigationEventArgs const&)
	{
		const auto& n{ this->rootNavView() };
		const auto& f{ this->ContentFrame() };
		n.IsBackEnabled(f.CanGoBack());

		if (const auto& name{ f.SourcePageType().Name }; name == xaml_typename<simbolmudah_ui::SettingsPage>().Name)
		{
			n.SelectedItem(n.SettingsItem().as<NavigationViewItem>());
			n.Header(box_value(L"Settings"));
		}
		else if (name != L"")
		{
			for (const auto&& i : n.MenuItems())
			{
				const auto& item{ i.try_as<NavigationViewItem>() };
				if (item && unbox_value_or<hstring>(item.Tag(), L"") == name)
				{
					n.SelectedItem(item);
					n.Header(item.Content());
				}
			}
		}
	}

	void MainWindow::ContentFrame_NavigationFailed(IInspectable const&, NavigationFailedEventArgs const& e)
	{
		throw hresult_error(E_FAIL, L"Failed to load Page " + e.SourcePageType().Name);
	}

	void MainWindow::NavigationViewControl_Loaded(IInspectable const&, RoutedEventArgs const&)
	{
		this->NavigateInternal(xaml_typename<simbolmudah_ui::HomePage>(), EntranceNavigationTransitionInfo());
	}

	void MainWindow::NavigationViewControl_ItemInvoked(NavigationView const&, NavigationViewItemInvokedEventArgs const& args)
	{
		if (args.IsSettingsInvoked())
		{
			this->NavigateInternal(xaml_typename<simbolmudah_ui::SettingsPage>(), args.RecommendedNavigationTransitionInfo());
		}
		else if (args.InvokedItemContainer())
		{
			this->NavigateInternal(
				TypeName{ .Name = unbox_value<hstring>(args.InvokedItemContainer().Tag()), .Kind = TypeKind::Metadata },
				args.RecommendedNavigationTransitionInfo()
			);
		}
	}

	void MainWindow::NavigationViewControl_BackRequested(NavigationView const&, NavigationViewBackRequestedEventArgs const&)
	{
		if (const auto& f{ this->ContentFrame() }; f.CanGoBack())
		{
			if (const auto& n{ this->rootNavView() }; !n.IsPaneOpen() && n.DisplayMode() == NavigationViewDisplayMode::Expanded)
			{
				f.GoBack();
			}
		}
	}

	void MainWindow::Window_SizeChanged(IInspectable const&, WindowSizeChangedEventArgs const& args)
	{
		if (const auto& n{ this->rootNavView() }; args.Size().Width <= n.CompactModeThresholdWidth())
		{
			n.PaneDisplayMode(NavigationViewPaneDisplayMode::Auto);
		}
		else
		{
			n.PaneDisplayMode(NavigationViewPaneDisplayMode::Top);
		}
	}

	void MainWindow::NavigateInternal(TypeName const& navPageType, NavigationTransitionInfo const& transitionInfo)
	{
		if (navPageType.Name != L"")
		{
			if (const auto& contentFrame{ this->ContentFrame() }; contentFrame.CurrentSourcePageType().Name != navPageType.Name)
			{
				contentFrame.Navigate(navPageType, nullptr, transitionInfo);
			}
		}
	}
}

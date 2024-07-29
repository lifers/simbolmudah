#include "pch.hpp"
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

    MainWindow::MainWindow(uint8_t page) : main_thread{ apartment_context() }
    {
        if (page == 1)
        {
            this->Activated([weak_this{ this->get_weak() }](auto&&, auto&&) {
                if (const auto& self{ weak_this.get() }; self)
                {
                    self->rootNavView().Loaded([self](auto&&, auto&&) {
                        self->NavigateInternal(L"simbolmudah_ui.SettingsPage", EntranceNavigationTransitionInfo());
                    });
                }
            });
        }
        else
        {
            this->Activated([weak_this{ this->get_weak() }](auto&&, auto&&) {
                if (const auto& self{ weak_this.get() }; self)
                {
                    self->rootNavView().Loaded([self](auto&&, auto&&) {
                        self->NavigateInternal(L"simbolmudah_ui.HomePage", EntranceNavigationTransitionInfo());
                    });
                }
            });
        }
    }

    void MainWindow::ContentFrame_Navigated(IInspectable const&, NavigationEventArgs const&)
    {
        const auto& n{ this->rootNavView() };
        const auto& f{ this->ContentFrame() };
        n.IsBackEnabled(f.CanGoBack());

        if (const auto& name{ f.SourcePageType().Name }; name == L"simbolmudah_ui.SettingsPage")
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
        this->NavigateInternal(L"simbolmudah_ui.HomePage", EntranceNavigationTransitionInfo());
    }

    void MainWindow::NavigationViewControl_ItemInvoked(NavigationView const&, NavigationViewItemInvokedEventArgs const& args)
    {
        if (args.IsSettingsInvoked())
        {
            this->NavigateInternal(L"simbolmudah_ui.SettingsPage", args.RecommendedNavigationTransitionInfo());
        }
        else if (args.InvokedItemContainer())
        {
            this->NavigateInternal(
                unbox_value<hstring>(args.InvokedItemContainer().Tag()),
                args.RecommendedNavigationTransitionInfo());
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

    void MainWindow::NavigateInternal(hstring const& navPageName, NavigationTransitionInfo const& transitionInfo)
    {
        if (navPageName != L"")
        {
            if (const auto& contentFrame{ this->ContentFrame() }; contentFrame.CurrentSourcePageType().Name != navPageName)
            {
                contentFrame.Navigate({ navPageName, TypeKind::Metadata }, nullptr, transitionInfo);
            }
        }
    }

    void MainWindow::OpenSettings()
    {
        this->NavigateInternal(L"simbolmudah_ui.SettingsPage", EntranceNavigationTransitionInfo());
    }
}

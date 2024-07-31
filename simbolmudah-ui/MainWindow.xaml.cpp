#include "pch.hpp"
#include "MainWindow.xaml.h"
#if __has_include("MainWindow.g.cpp")
#include "MainWindow.g.cpp"
#endif

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Microsoft::UI::Xaml;
    using namespace Navigation;
    using namespace Controls;
    using namespace Media::Animation;
    using namespace Windows::UI::Xaml::Interop;

    MainWindow::MainWindow(
        SequenceDefinition const& seqdef,
        simbolmudah_ui::AppManager const& appManager,
        NotifyIcon const& notifyIcon,
        uint8_t page) :
        main_thread{ apartment_context() },
        sequenceDefinition{ seqdef },
        appManager{ appManager }
    {
        if (this->appManager.NotifyIconEnabled() && notifyIcon)
        {
            this->openSettingsRevoker = notifyIcon.OnOpenSettings(
                auto_revoke, { this->get_weak(), &MainWindow::OnOpenSettings });
        }

        this->ExtendsContentIntoTitleBar(true);
        this->Closed({ this, &MainWindow::OnClosed });
    }

    /// <summary>
    /// Subscribe to the OpenSettings event if the NotifyIcon is enabled,
    /// otherwise unsubscribe from the event.
    /// </summary>
    /// <param name="notifyIcon">reference to the NotifyIcon (could be null)</param>
    void MainWindow::UpdateOpenSettings(NotifyIcon const& notifyIcon)
    {
        if (this->appManager.NotifyIconEnabled() && notifyIcon)
        {
            this->openSettingsRevoker = notifyIcon.OnOpenSettings(
                auto_revoke, { this->get_weak(), &MainWindow::OnOpenSettings });
        }
        else
        {
            this->openSettingsRevoker.revoke();
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
        this->NavigateToSearch(EntranceNavigationTransitionInfo());
    }

    void MainWindow::NavigationViewControl_ItemInvoked(NavigationView const&, NavigationViewItemInvokedEventArgs const& args)
    {
        if (args.IsSettingsInvoked())
        {
            this->NavigateToSettings(args.RecommendedNavigationTransitionInfo());
        }
        else if (args.InvokedItemContainer())
        {
            const auto& n{ unbox_value<hstring>(args.InvokedItemContainer().Tag()) };
            if (n == L"simbolmudah_ui.SearchPage")
            {
                this->NavigateToSearch(args.RecommendedNavigationTransitionInfo());
            }
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

    void MainWindow::NavigateToSearch(NavigationTransitionInfo const& transitionInfo)
    {
        if (const auto& f{ this->ContentFrame() }; f.CurrentSourcePageType().Name != L"simbolmudah_ui.SearchPage")
        {
            f.Navigate({ L"simbolmudah_ui.SearchPage", TypeKind::Metadata }, this->sequenceDefinition, transitionInfo);
        }
    }

    void MainWindow::NavigateToSettings(NavigationTransitionInfo const& transitionInfo)
    {
        if (const auto& f{ this->ContentFrame() }; f.CurrentSourcePageType().Name != L"simbolmudah_ui.SettingsPage")
        {
            f.Navigate({ L"simbolmudah_ui.SettingsPage", TypeKind::Metadata }, this->appManager, transitionInfo);
        }
    }

    fire_and_forget MainWindow::OnOpenSettings(NotifyIcon const&, bool)
    {
        co_await this->main_thread;
        this->Activate();
        this->NavigateToSettings(EntranceNavigationTransitionInfo());
    }

    void MainWindow::OnClosed(IInspectable const&, WindowEventArgs const&)
    {
        this->openSettingsRevoker.revoke();
    }
}

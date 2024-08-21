#include "pch.hpp"
#include "MainWindow.xaml.h"
#if __has_include("MainWindow.g.cpp")
#include "MainWindow.g.cpp"
#endif
#include <CommCtrl.h>
#include <Microsoft.UI.Xaml.Window.h>
#include <wil/cppwinrt_helpers.h>


namespace
{
    // Limit the minimum window size to 640x480 times the DPI scale factor.
    LRESULT MinimumSizeSubclass(
        HWND hwnd,
        UINT message,
        WPARAM wParam,
        LPARAM lParam,
        UINT_PTR,
        DWORD_PTR
    ) {
        if (message == WM_GETMINMAXINFO)
        {
            const auto mmi{ reinterpret_cast<MINMAXINFO*>(lParam) };
            const auto dpi{ ::GetDpiForWindow(hwnd) };
            mmi->ptMinTrackSize.x = 640u * dpi / 96u;
            mmi->ptMinTrackSize.y = 480u * dpi / 96u;
            return 0;
        }
        else
        {
            return ::DefSubclassProc(hwnd, message, wParam, lParam);
        }
    }
}

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
        NotifyIcon const& notifyIcon) :
        sequenceDefinition{ seqdef }, appManager{ appManager }
    {
        if (this->appManager.NotifyIconEnabled() && notifyIcon)
        {
            this->openSettingsRevoker = notifyIcon.OnOpenSettings(
                auto_revoke, { this->get_weak(), &MainWindow::OnOpenSettings });
        }

        this->ExtendsContentIntoTitleBar(true);
        this->AppWindow().Resize({ 800, 600 });
        this->Closed({ this, &MainWindow::OnClosed });
        this->SetMinimumWindowSize();
    }

    /// <summary>
    /// Set the minimum window size using subclassing.
    /// </summary>
    void MainWindow::SetMinimumWindowSize()
    {
        const auto windowNative{ this->m_inner.as<::IWindowNative>() };
        HWND hwnd{};
        check_hresult(windowNative->get_WindowHandle(&hwnd));
        check_bool(::SetWindowSubclass(hwnd, MinimumSizeSubclass, 0, 0));
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
        using enum NavigationViewPaneDisplayMode;
        if (const auto& n{ this->rootNavView() }; args.Size().Width <= n.CompactModeThresholdWidth())
        {
            n.PaneDisplayMode(Auto);
        }
        else
        {
            n.PaneDisplayMode(Top);
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
        co_await wil::resume_foreground(this->DispatcherQueue());
        this->Activate();
        this->NavigateToSettings(EntranceNavigationTransitionInfo());
    }

    void MainWindow::OnClosed(IInspectable const&, WindowEventArgs const&)
    {
        this->openSettingsRevoker.revoke();
    }
}

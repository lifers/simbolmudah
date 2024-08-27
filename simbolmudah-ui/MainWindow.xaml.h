#pragma once

#include "MainWindow.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct MainWindow : MainWindowT<MainWindow>
    {
        explicit MainWindow(
            LibSimbolMudah::SequenceDefinition const& seqdef,
            simbolmudah_ui::AppManager const& appManager,
            LibSimbolMudah::NotifyIcon const& notifyIcon);
        MainWindow(MainWindow const&) = delete;
        MainWindow& operator=(MainWindow const&) = delete;

        void UpdateOpenSettings(LibSimbolMudah::NotifyIcon const& notifyIcon);

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
        void NavigateToSettings(Microsoft::UI::Xaml::Media::Animation::NavigationTransitionInfo const& transitionInfo);
        void SetSequenceDefinition(LibSimbolMudah::SequenceDefinition const& seqdef);

    private:
        void NavigateToSearch(Microsoft::UI::Xaml::Media::Animation::NavigationTransitionInfo const& transitionInfo);
        fire_and_forget OnOpenSettings(LibSimbolMudah::NotifyIcon const&, bool);
        void OnClosed(IInspectable const&, Microsoft::UI::Xaml::WindowEventArgs const&);
        void SetMinimumWindowSize();

        LibSimbolMudah::SequenceDefinition sequenceDefinition{ nullptr };
        const simbolmudah_ui::AppManager appManager;
        LibSimbolMudah::NotifyIcon::OnOpenSettings_revoker openSettingsRevoker;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct MainWindow : MainWindowT<MainWindow, implementation::MainWindow>
    {
    };
}

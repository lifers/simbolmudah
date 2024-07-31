#pragma once

#include "SettingsPage.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct SettingsPage : SettingsPageT<SettingsPage>
    {
        simbolmudah_ui::AppManager ViewModel() const;
        void OnNavigatedTo(Microsoft::UI::Xaml::Navigation::NavigationEventArgs const& e);
        void OnSaveClick(IInspectable const&, Microsoft::UI::Xaml::RoutedEventArgs const&);
        void OnCancelClick(IInspectable const&, Microsoft::UI::Xaml::RoutedEventArgs const&);

    private:
        simbolmudah_ui::AppManager viewModel{ nullptr };
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct SettingsPage : SettingsPageT<SettingsPage, implementation::SettingsPage>
    {
    };
}

#pragma once

#include "SettingsPage.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct SettingsPage : SettingsPageT<SettingsPage>
    {
        SettingsPage();
        SettingsPage(SettingsPage const&) = delete;
        SettingsPage(SettingsPage&&) = delete;
        SettingsPage& operator=(SettingsPage const&) = delete;

        simbolmudah_ui::AppManager ViewModel() const;
        void OnNavigatedTo(Microsoft::UI::Xaml::Navigation::NavigationEventArgs const& e);
        void OnNavigatingFrom(Microsoft::UI::Xaml::Navigation::NavigatingCancelEventArgs const& e);
        void OpenTutorial(IInspectable const& sender, Microsoft::UI::Xaml::RoutedEventArgs const& e) const;

    private:
        fire_and_forget OnSettingsChanged(
            IInspectable const& sender,
            Microsoft::UI::Xaml::Data::PropertyChangedEventArgs const& e);
        void ChangePopupSetting(bool isEnabled);

        const Microsoft::UI::Xaml::ResourceDictionary resources;
        const Microsoft::UI::Xaml::Media::Brush disabledTextColor;
        const Microsoft::UI::Xaml::Media::Brush enabledTextColor;
        simbolmudah_ui::AppManager viewModel;
        simbolmudah_ui::AppManager::PropertyChanged_revoker settingsChangedRevoker;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct SettingsPage : SettingsPageT<SettingsPage, implementation::SettingsPage>
    {
    };
}

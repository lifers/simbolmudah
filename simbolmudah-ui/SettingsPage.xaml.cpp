#include "pch.hpp"
#include "SettingsPage.xaml.h"
#if __has_include("SettingsPage.g.cpp")
#include "SettingsPage.g.cpp"
#endif

namespace winrt::simbolmudah_ui::implementation
{
    using namespace Microsoft::UI::Xaml;
    using namespace Navigation;

    simbolmudah_ui::AppManager SettingsPage::ViewModel() const { return this->viewModel; }

    void SettingsPage::OnNavigatedTo(NavigationEventArgs const& e)
    {
        const auto& appManager{ e.Parameter().as<simbolmudah_ui::AppManager>() };
        this->viewModel = appManager;
    }

    void SettingsPage::OnSaveClick(IInspectable const&, RoutedEventArgs const&)
    {
        this->viewModel.SaveSettings({
            .HookEnabled = this->HookSwitch().IsOn(),
            .NotifyIconEnabled = this->NotifyIconSwitch().IsOn(),
            .MainWindowOpened = this->MainWindowSwitch().IsOn(),
        });
    }

    void SettingsPage::OnCancelClick(IInspectable const&, RoutedEventArgs const&)
    {
        this->HookSwitch().IsOn(this->viewModel.HookEnabled());
        this->NotifyIconSwitch().IsOn(this->viewModel.NotifyIconEnabled());
        this->MainWindowSwitch().IsOn(this->viewModel.MainWindowOpened());
    }
}

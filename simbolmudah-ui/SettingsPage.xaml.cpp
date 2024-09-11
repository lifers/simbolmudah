#include "pch.hpp"
#include "SettingsPage.xaml.h"
#if __has_include("SettingsPage.g.cpp")
#include "SettingsPage.g.cpp"
#endif
#include <winrt/in_app_tutorial.h>
#include <wil/cppwinrt_helpers.h>

namespace winrt::simbolmudah_ui::implementation
{
    using namespace Microsoft::UI::Xaml;
    using namespace Data;
    using namespace Media;
    using namespace Navigation;

    SettingsPage::SettingsPage() :
        viewModel{ nullptr }, resources{ Application::Current().Resources() },
        disabledTextColor{ resources.Lookup(box_value(L"TextFillColorDisabledBrush")).as<Brush>() },
        enabledTextColor{ resources.Lookup(box_value(L"TextFillColorPrimaryBrush")).as<Brush>() } {}

    simbolmudah_ui::AppManager SettingsPage::ViewModel() const { return this->viewModel; }

    void SettingsPage::OnNavigatedTo(NavigationEventArgs const& e)
    {
        const auto& appManager{ e.Parameter().as<simbolmudah_ui::AppManager>() };
        this->viewModel = appManager;
        this->settingsChangedRevoker = this->viewModel.PropertyChanged(
            auto_revoke, { this->get_weak(), &SettingsPage::OnSettingsChanged });
        this->ChangePopupSetting(this->viewModel.HookEnabled());
    }

    void SettingsPage::OnNavigatingFrom(NavigatingCancelEventArgs const&)
    {
        this->settingsChangedRevoker.revoke();
        this->viewModel = nullptr;
    }

    void SettingsPage::OpenTutorial(IInspectable const&, RoutedEventArgs const&) const
    {
        const auto& dialog{ in_app_tutorial::TutorialDialog::GetDialog() };
        dialog.XamlRoot(this->XamlRoot());
        dialog.ShowAsync();
    }

    fire_and_forget SettingsPage::OnSettingsChanged(IInspectable const&, PropertyChangedEventArgs const& e)
    {
        const auto propertyName{ e.PropertyName() };
        co_await wil::resume_foreground(this->DispatcherQueue());

        if (propertyName == L"" || propertyName == L"HookEnabled")
        {
            this->ChangePopupSetting(this->viewModel.HookEnabled());
        }
    }

    void SettingsPage::ChangePopupSetting(bool isEnabled)
    {
        this->PopupIcon().Foreground(isEnabled ? this->enabledTextColor : this->disabledTextColor);
        this->PopupText().Foreground(isEnabled ? this->enabledTextColor : this->disabledTextColor);
    }
}

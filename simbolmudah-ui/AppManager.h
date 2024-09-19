#pragma once
#include "AppManager.g.h"
#include <wil/winrt.h>
#include <wil/cppwinrt_authoring.h>

namespace winrt::simbolmudah_ui::implementation
{
    struct AppManager : AppManagerT<AppManager>, wil::notify_property_changed_base<AppManager>
    {
        explicit AppManager(Windows::Storage::ApplicationDataContainer const& localSettings);
        AppManager(AppManager const&) = delete;
        AppManager& operator=(AppManager const&) = delete;

        bool HookEnabled() const noexcept { return this->hookEnabled; }
        void HookEnabled(bool value);
        bool UseHookPopup() const noexcept { return this->useHookPopup; }
        void UseHookPopup(bool value);
        bool NotifyIconEnabled() const noexcept { return this->notifyIconEnabled; }
        void NotifyIconEnabled(bool value);
        bool MainWindowOpened() const noexcept { return this->mainWindowOpened; }
        void MainWindowOpened(bool value);
        bool FirstInstall() const noexcept { return !this->localSettings.Values().HasKey(L"firstInstall"); }
        void FirstInstall(bool value) { this->localSettings.Values().Insert(L"firstInstall", box_value(value)); }

    private:
        const apartment_context main_thread{ apartment_context() };
        const Windows::Storage::ApplicationDataContainer localSettings;
        bool hookEnabled{ false };
        bool useHookPopup{ false };
        bool notifyIconEnabled{ true };
        bool mainWindowOpened{ true };
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct AppManager : AppManagerT<AppManager, implementation::AppManager>
    {
    };
}
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

        fire_and_forget SaveSettings(simbolmudah_ui::SettingsObject settings);

        WIL_NOTIFYING_PROPERTY(bool, HookEnabled, false);
        WIL_NOTIFYING_PROPERTY(bool, UseHookPopup, false);
        WIL_NOTIFYING_PROPERTY(bool, NotifyIconEnabled, true);
        WIL_NOTIFYING_PROPERTY(bool, MainWindowOpened, true);

        wil::untyped_event<LibSimbolMudah::NotifyIcon> NotifyIconChanged;

    private:
        const apartment_context main_thread;
        const Windows::Storage::ApplicationDataContainer localSettings;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct AppManager : AppManagerT<AppManager, implementation::AppManager>
    {
    };
}
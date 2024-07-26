#pragma once

#include "NotifyIconPopup.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct NotifyIconPopup : NotifyIconPopupT<NotifyIconPopup>
    {
        NotifyIconPopup();
        NotifyIconPopup(const NotifyIconPopup&) = delete;
        NotifyIconPopup& operator=(const NotifyIconPopup&) = delete;

        void MenuFlyout_Closed(IInspectable const& sender, IInspectable const& e);
        void MenuFlyout_Opened(IInspectable const& sender, IInspectable const& e);
        void InvisibleRect_LostFocus(IInspectable const& sender, Microsoft::UI::Xaml::RoutedEventArgs const& e);

    private:
        fire_and_forget ShowFlyout(LibSimbolMudah::NotifyIcon const& sender, Windows::Graphics::PointInt32 args);

        const apartment_context main_thread;
        const LibSimbolMudah::NotifyIcon notifyIcon;
        const LibSimbolMudah::NotifyIcon::OnSelected_revoker selectedToken;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct NotifyIconPopup : NotifyIconPopupT<NotifyIconPopup, implementation::NotifyIconPopup>
    {
    };
}

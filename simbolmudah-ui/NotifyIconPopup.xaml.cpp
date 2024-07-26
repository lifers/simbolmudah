#include "pch.hpp"
#include "NotifyIconPopup.xaml.h"
#if __has_include("NotifyIconPopup.g.cpp")
#include "NotifyIconPopup.g.cpp"
#endif

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Microsoft::UI;
    using namespace Windowing;
    using namespace Xaml;
    using namespace Controls;
    using namespace Primitives;
    using namespace Windows::Graphics;
    using namespace std::chrono_literals;

    NotifyIconPopup::NotifyIconPopup() :
        main_thread{ apartment_context() },
        selectedToken{ this->notifyIcon.OnSelected(auto_revoke, { this->get_weak(), &NotifyIconPopup::ShowFlyout }) }
    {
        const auto presenter{ OverlappedPresenter::CreateForContextMenu() };
        presenter.IsAlwaysOnTop(true);

        const auto& appWindow{ this->AppWindow() };
        appWindow.SetPresenter(presenter);
        //appWindow.IsShownInSwitchers(false);
        appWindow.Resize({ 1, 1 });
        const auto hwnd{ GetWindowFromWindowId(appWindow.Id()) };
        WINRT_VERIFY(hwnd != nullptr);

        // Get the current window style
        auto dwStyle{ GetWindowLongPtrW(hwnd, GWL_STYLE) };

        // Remove unwanted styles and add WS_POPUP
        dwStyle &= ~(WS_OVERLAPPEDWINDOW | WS_CAPTION | WS_THICKFRAME | WS_BORDER |
                     WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU | WS_VISIBLE);
        dwStyle |= WS_POPUP;

        // Set the new window style
        SetWindowLongPtrW(hwnd, GWL_STYLE, dwStyle);

        //auto dwexStyle{ GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
        
        //// Remove unwanted styles and add WS_POPUP
        //dwexStyle &= ~(WS_EX_TRANSPARENT)
        //dwexStyle |= WS_POPUP;

        // Set the new window style
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, WS_EX_TRANSPARENT | WS_EX_LAYERED);
    }

    void NotifyIconPopup::MenuFlyout_Closed(IInspectable const&, IInspectable const&)
    {
        const auto& appWindow{ this->AppWindow() };
        appWindow.Hide();
    }

    void NotifyIconPopup::MenuFlyout_Opened(IInspectable const&, IInspectable const&)
    {
        this->ShareButton().Focus(FocusState::Keyboard);
    }

    void NotifyIconPopup::InvisibleRect_LostFocus(IInspectable const&, RoutedEventArgs const&)
    {
        const auto& appWindow{ this->AppWindow() };
        appWindow.Hide();
    }
    
    fire_and_forget NotifyIconPopup::ShowFlyout(NotifyIcon const& sender, PointInt32 args)
    {
        co_await this->main_thread;
        const auto& appWindow{ this->AppWindow() };
        //appWindow.MoveInZOrderAtTop();

        appWindow.MoveAndResize({ args.X, args.Y, 0, 0 });
        //appWindow.Show(true);
        this->Activate();
        FlyoutBase::ShowAttachedFlyout(this->InvisibleRect());
        //SetFocus(GetWindowFromWindowId(this->AppWindow().Id()));

        /*co_await 5s;
        co_await this->main_thread;
        appWindow.Hide();*/
    }
}

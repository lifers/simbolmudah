#include "pch.hpp"
#include "PopupWindow.xaml.h"
#if __has_include("PopupWindow.g.cpp")
#include "PopupWindow.g.cpp"
#endif
#include <Microsoft.UI.Xaml.Window.h>
#include <winrt/Microsoft.UI.Dispatching.h>
#include <wil/cppwinrt_helpers.h>

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace
{
    using namespace winrt;

    VARIANT EmptyVariant()
    {
        VARIANT v{};
        VariantInit(&v);
        return v;
    }

    VARIANT SelfVariant()
    {
        VARIANT v{ EmptyVariant() };
        v.vt = VT_I4;
        v.lVal = CHILDID_SELF;
        return v;
    }

    constexpr POINT ZERO_POINT{};
    constexpr bool operator==(const POINT& lhs, const POINT& rhs) noexcept
    {
        return lhs.x == rhs.x && lhs.y == rhs.y;
    }

    // Try to use UIAutomation to get the caret position
    POINT TryUIAutomation(HWND hwnd)
    {
        // Create a IUIAutomation instance
        const auto rootobj{ create_instance<IUIAutomation>(CLSID_CUIAutomation) };

        // Get the root element of the window
        com_ptr<IUIAutomationElement> element{ nullptr };
        check_hresult(rootobj->ElementFromHandle(hwnd, element.put()));

        // Check if the element supports the TextPattern2 interface
        auto v{ EmptyVariant() };
        element->GetCurrentPropertyValue(UIA_IsTextPattern2AvailablePropertyId, &v);
        if (!v.boolVal) { return ZERO_POINT; }

        // Get the TextPattern2 interface
        const auto textPattern{ capture<IUIAutomationTextPattern2>(
                    element, &IUIAutomationElement::GetCurrentPatternAs, UIA_TextPattern2Id) };

        // Get the caret range
        BOOL isCaretActive{};
        com_ptr<IUIAutomationTextRange> range{ nullptr };
        check_hresult(textPattern->GetCaretRange(&isCaretActive, range.put()));

        // Get the bounding rectangles of the caret
        auto arr{ check_pointer(::SafeArrayCreateVector(VT_R8, 1, 0)) };
        check_hresult(range->GetBoundingRectangles(&arr));
        RECT* r{ nullptr };
        int rectCount{ 0 };
        check_hresult(rootobj->SafeArrayToRectNativeArray(arr, &r, &rectCount));
        check_hresult(::SafeArrayDestroy(arr)); // Clean up the SafeArray

        // Return the first rectangle if available
        if (r && rectCount > 0)
        {
            return { .x = r[0].left, .y = r[0].top };
        }
        else
        {
            return ZERO_POINT;
        }
    }

    // Try to use the Microsoft Active Accessibility API to get the caret position
    POINT TryMSAA(HWND hwnd)
    {
        const auto pAcc{ capture<IAccessible>(::AccessibleObjectFromWindow, hwnd, static_cast<DWORD>(OBJID_CARET)) };

        // Get the caret position in screen coordinates
        RECT r{};
        check_hresult(pAcc->accLocation(&r.left, &r.top, &r.right, &r.bottom, SelfVariant()));

        if (r.right > 0 || r.bottom > 0 || r.top > 0 || r.bottom > 0)
        {
            POINT p{ .x = r.left, .y = r.top };
            check_bool(::ClientToScreen(hwnd, &p));
            return p;
        }
        else
        {
            return ZERO_POINT;
        }
    }

    // Get the location of the active text cursor (caret)
    POINT GetCaretPosition()
    {
        // Get the window with keyboard focus
        GUITHREADINFO gti{ .cbSize = sizeof(GUITHREADINFO) };
        check_bool(::GetGUIThreadInfo(NULL, &gti));

        if (gti.hwndCaret)
        {
            // Get the caret position in screen coordinates
            POINT caretCoord{ .x = gti.rcCaret.right, .y = gti.rcCaret.top };
            check_bool(::ClientToScreen(gti.hwndCaret, &caretCoord));
            return caretCoord;
        }
        else if (gti.hwndFocus)
        {
            auto r{ TryUIAutomation(gti.hwndFocus) };
            if (r == ZERO_POINT) { r = TryMSAA(gti.hwndFocus); }
            if (r != ZERO_POINT) { return r; }
        }

        // If there is no caret, return the mouse position
        POINT mousePos{};
        check_bool(::GetCursorPos(&mousePos));
        return mousePos;
    }
}

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Microsoft::UI;
    using namespace Xaml;
    using namespace Controls;
    using namespace Windowing;
    using namespace std::chrono_literals;

    PopupWindow::PopupWindow(KeyboardTranslator const& translator, KeyboardHook const& hook, SequenceDefinition const& definition) :
        translator{ translator }, hook{ hook },
        keyTranslatedToken{ this->translator.OnKeyTranslated(auto_revoke, { this->get_weak(), &PopupWindow::OnKeyTranslated }) },
        stateChangedToken{ this->hook.OnStateChanged(auto_revoke, { this->get_weak(), &PopupWindow::OnStateChanged }) },
        defaultPage{ Page() }, sequencePopup{ definition }, searchPopup{ hook, definition }
    {
        const auto presenter{ OverlappedPresenter::CreateForContextMenu() };
        presenter.IsAlwaysOnTop(true);
        
        const auto& appWindow{ this->AppWindow() };
        appWindow.SetPresenter(presenter);
        appWindow.Hide();

        const auto textBlock{ TextBlock() };
        textBlock.Text(L"Press a key to start.");
        textBlock.HorizontalAlignment(HorizontalAlignment::Center);
        textBlock.VerticalAlignment(VerticalAlignment::Center);
        this->defaultPage.Content(textBlock);
        this->Content(this->defaultPage);
    }

    fire_and_forget PopupWindow::OnKeyTranslated(KeyboardTranslator const&, hstring const& message) const
    {
        const auto key{ message };
        co_await wil::resume_foreground(this->DispatcherQueue());
        this->sequencePopup.Sequence().Append(key);
        this->sequencePopup.FindPotentialPrefix();
    }

    fire_and_forget PopupWindow::OnStateChanged(KeyboardHook const&, uint8_t state) const
    {
        co_await wil::resume_foreground(this->DispatcherQueue());
        switch (state)
        {
        case 0: // Idle
            this->AppWindow().Hide();
            co_return;
        case 2: // ComposeKeyupFirst
            this->Content(this->defaultPage);
            this->DrawWindow();
            co_return;
        case 4: // SequenceMode
            this->sequencePopup.Sequence().Clear();
            this->Content(this->sequencePopup);
            co_return;
        case 5: // SearchMode
            this->searchPopup.SearchResults().Clear();
            this->Content(this->searchPopup);
            this->AppWindow().Show(true);
            co_return;
        }
    }

    fire_and_forget PopupWindow::DrawWindow() const
    {
        co_await resume_background();
        const auto pos{ GetCaretPosition() };
        const auto dpi{ this->GetDpi() };

        co_await wil::resume_foreground(this->DispatcherQueue());
        const auto& appWindow{ this->AppWindow() };
        appWindow.MoveAndResize({ .X = pos.x, .Y = pos.y, .Width = 400 * dpi / 96, .Height = 100 * dpi / 96 });
        appWindow.Show(false);
    }

    int32_t PopupWindow::GetDpi() const
    {
        const auto windowNative{ this->m_inner.as<::IWindowNative>() };
        HWND hwnd{};
        check_hresult(windowNative->get_WindowHandle(&hwnd));
        return static_cast<int32_t>(::GetDpiForWindow(hwnd));
    }
}

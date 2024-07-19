#include "pch.hpp"
#include "BlankWindow.xaml.h"
#if __has_include("BlankWindow.g.cpp")
#include "BlankWindow.g.cpp"
#endif

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace
{
	using namespace winrt;

	VARIANT SelfVariant()
	{
		VARIANT v{};
		VariantInit(&v);
		v.vt = VT_I4;
		v.lVal = CHILDID_SELF;
		return v;
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
			// Try to get the caret position in the window with focus
			if (IAccessible* pAcc{ nullptr };
				S_OK == ::AccessibleObjectFromWindow(
					gti.hwndFocus, static_cast<DWORD>(OBJID_CARET), IID_IAccessible, reinterpret_cast<void**>(&pAcc))
				&& check_pointer(pAcc))
			{
				// Get the caret position in screen coordinates
				RECT r{};
				check_hresult(pAcc->accLocation(
					&r.left, &r.top, &r.right, &r.bottom, SelfVariant()));
				WINRT_VERIFY(pAcc->Release() == 0);

				if (r.right > 0 || r.bottom > 0 || r.top > 0 || r.bottom > 0)
				{
					POINT p{ .x = r.left, .y = r.top };
					check_bool(::ClientToScreen(gti.hwndFocus, &p));
					return p;
				}
			}
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

	BlankWindow::BlankWindow(KeyboardTranslator const& translator, KeyboardHook const& hook, SequenceDefinition const& definition) :
		translator{ translator }, hook{ hook }, main_thread{ apartment_context() },
		keyTranslatedToken{ this->translator.OnKeyTranslated(auto_revoke, { this->get_weak(), &BlankWindow::OnKeyTranslated }) },
		stateChangedToken{ this->hook.OnStateChanged(auto_revoke, { this->get_weak(), &BlankWindow::OnStateChanged }) },
		defaultPage{ Page() }, sequencePopup{ definition }
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

	fire_and_forget BlankWindow::OnKeyTranslated(KeyboardTranslator const&, hstring const& message) const
	{
		const auto key{ message };
		co_await this->main_thread;
		this->sequencePopup.Sequence().Append(key);
		this->sequencePopup.FindPotentialPrefix();
	}

	fire_and_forget BlankWindow::OnStateChanged(KeyboardHook const&, uint8_t state) const
	{
		co_await this->main_thread;
		switch (state)
		{
		case 0: // Idle
			this->AppWindow().Hide();
			co_return;
		case 2: // ComposeKeyupFirst
		{
			this->Content(this->defaultPage);
			this->DrawWindow();
			co_return;
		}
		case 4: // SequenceMode
			this->sequencePopup.Sequence().Clear();
			this->Content(this->sequencePopup);
			co_return;
		}
	}

	void BlankWindow::DrawWindow() const
	{
		const auto& appWindow{ this->AppWindow() };
		const auto dpi{ static_cast<int32_t>(::GetDpiForWindow(GetWindowFromWindowId(appWindow.Id()))) };
		const auto pos{ GetCaretPosition() };
		appWindow.MoveAndResize({ .X = pos.x, .Y = pos.y, .Width = 400 * dpi / 96, .Height = 100 * dpi / 96 });
		appWindow.Show(false);
	}
}

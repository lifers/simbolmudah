#include "pch.hpp"
#include "BlankWindow.xaml.h"
#if __has_include("BlankWindow.g.cpp")
#include "BlankWindow.g.cpp"
#endif

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

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
		const auto& appWindow{ this->AppWindow() };
		appWindow.Resize({ 400, 100 });
		appWindow.SetPresenter(OverlappedPresenter::CreateForContextMenu());
		appWindow.Hide();

		const auto textBlock{ TextBlock() };
		textBlock.Text(L"Start composing, or press ESC to exit");
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
			const auto& appWindow{ this->AppWindow() };
			appWindow.MoveInZOrderAtTop();
			appWindow.Show();
			co_return;
		}
		case 4: // SequenceMode
			this->sequencePopup.Sequence().Clear();
			this->Content(this->sequencePopup);
			co_return;
		}
	}
}

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
	using namespace Windowing;
	using namespace Windows::Foundation;
	using namespace std::chrono_literals;

	BlankWindow::BlankWindow(KeyboardTranslator const& translator) :
		app{ Application::Current().as<App>() }, translator{ translator }
	{
		const auto& appWindow{ this->AppWindow() };
		appWindow.Resize({ 400, 300 });
		appWindow.SetPresenter(OverlappedPresenter::CreateForContextMenu());
		appWindow.Hide();

		this->showResultsToken = this->translator.OnTranslated(
			TypedEventHandler<KeyboardTranslator, hstring>::TypedEventHandler(this->get_weak(), &BlankWindow::ShowResult)
		);
	}

	BlankWindow::~BlankWindow()
	{
		this->translator.OnTranslated(this->showResultsToken);
	}

    fire_and_forget BlankWindow::ShowResult(KeyboardTranslator const&, hstring const& message)
	{
		const auto result{ message };
		co_await this->app->main_thread;
		this->resultBar().Message(result);
		this->resultBar().IsOpen(true);
		const auto& appWindow{ this->AppWindow() };
		appWindow.Show();
		co_await 5s;
		appWindow.Hide();
	}
}

#include "pch.hpp"
#include "HomePage.xaml.h"
#if __has_include("HomePage.g.cpp")
#include "HomePage.g.cpp"
#endif

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
	using namespace Windows;
	using namespace Foundation;
	using namespace Storage;
	using namespace Microsoft::UI::Xaml;
	using namespace Navigation;

	HomePage::HomePage() : app{ Application::Current().as<App>() } {}

	void HomePage::OnNavigatedTo(const NavigationEventArgs&)
	{
		this->showResultsToken = this->app->keyboardTranslator.OnTranslated(
			TypedEventHandler<KeyboardTranslator, hstring>::TypedEventHandler(this->get_weak(), &HomePage::ShowResult)
		);
	}

	void HomePage::OnNavigatingFrom(const NavigatingCancelEventArgs&)
	{
		this->app->keyboardTranslator.OnTranslated(this->showResultsToken);
	}

	void HomePage::OnUnloaded(const IInspectable&, const RoutedEventArgs&)
	{
		ApplicationData::Current().LocalSettings().Values().Insert(L"keyboardHookEnabled", box_value(this->app->hookState));	
	}

	void HomePage::HookEnabled(bool value)
	{
		this->app->hookState = value;
		this->app->OnHookToggle(value);
	}

	bool HomePage::HookEnabled() const
	{
		return this->app->hookState;
	}

	void HomePage::Button_Click(IInspectable const&, RoutedEventArgs const&)
	{
		this->buttonOne().Content(box_value(L"Clicked"));
	}

	fire_and_forget HomePage::InfoUpdater(const KeyboardHook&, const hstring& message)
	{
		const hstring result{ message };
		co_await this->app->main_thread;
		this->infoBar().Message(result);
		this->infoBar().IsOpen(true);
	}

	fire_and_forget HomePage::StateUpdater(const KeyboardHook&, const hstring& message)
	{
		const hstring result{ message };
		co_await this->app->main_thread;
		this->stateBar().Message(result);
		this->stateBar().IsOpen(true);
	}

	fire_and_forget HomePage::ShowResult(const KeyboardTranslator&, const hstring& message)
	{
		const hstring result{ message };
		co_await this->app->main_thread;
		this->resultBar().Message(result);
		this->resultBar().IsOpen(true);
	}
}

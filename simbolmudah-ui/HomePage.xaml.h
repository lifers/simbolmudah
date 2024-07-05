#pragma once

#include "HomePage.g.h"
#include "App.xaml.h"

namespace winrt::simbolmudah_ui::implementation
{
   struct HomePage : HomePageT<HomePage>
	{
		HomePage();
		HomePage(const HomePage&) = delete;
		HomePage& operator=(const HomePage&) = delete;
		void OnNavigatedTo(const Microsoft::UI::Xaml::Navigation::NavigationEventArgs&);
		void OnNavigatingFrom(const Microsoft::UI::Xaml::Navigation::NavigatingCancelEventArgs&);
		void OnUnloaded(const IInspectable&, const Microsoft::UI::Xaml::RoutedEventArgs&);
		void HookEnabled(bool value);
		bool HookEnabled() const;

	private:
		fire_and_forget InfoUpdater(const LibSimbolMudah::KeyboardHook&, const hstring& message);
		fire_and_forget StateUpdater(const LibSimbolMudah::KeyboardHook&, const hstring& message);
		fire_and_forget ShowResult(const LibSimbolMudah::KeyboardTranslator&, const hstring& message);

		const com_ptr<App> app;
		event_token showResultsToken;
		event_token infoUpdaterToken;
		event_token stateUpdaterToken;
   };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct HomePage : HomePageT<HomePage, implementation::HomePage>
    {
    };
}

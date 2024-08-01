#pragma once
#include "SearchPage.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct SearchPage : SearchPageT<SearchPage>
    {
        simbolmudah_ui::SearchPageViewModel MainViewModel() const;
        void SubmitSearch(
            Microsoft::UI::Xaml::Controls::AutoSuggestBox const& sender,
            Microsoft::UI::Xaml::Controls::AutoSuggestBoxTextChangedEventArgs const& e);
        void OnNavigatedTo(Microsoft::UI::Xaml::Navigation::NavigationEventArgs const& e);

    private:
        void CurrentSearch_Completed(
            Windows::Foundation::IAsyncAction const& asyncInfo,
            Windows::Foundation::AsyncStatus asyncStatus);

        simbolmudah_ui::SearchPageViewModel mainViewModel{ nullptr };
        Windows::Foundation::IAsyncAction currentSearch{ nullptr };
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct SearchPage : SearchPageT<SearchPage, implementation::SearchPage>
    {
    };
}

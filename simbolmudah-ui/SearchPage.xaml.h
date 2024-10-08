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
        void SetSequenceDefinition(LibSimbolMudah::SequenceDefinition const& seqdef);

    private:
        void CurrentSearch_Completed(
            Windows::Foundation::IAsyncAction const& asyncInfo,
            Windows::Foundation::AsyncStatus asyncStatus);

        simbolmudah_ui::SearchPageViewModel mainViewModel;
        Windows::Foundation::IAsyncAction currentSearch{ nullptr };
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct SearchPage : SearchPageT<SearchPage, implementation::SearchPage>
    {
    };
}

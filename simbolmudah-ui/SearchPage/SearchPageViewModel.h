#pragma once
#include "SearchPageViewModel.g.h"

namespace winrt::simbolmudah_ui::implementation
{   
    struct SearchPageViewModel : SearchPageViewModelT<SearchPageViewModel>
    {
        SearchPageViewModel();
        SearchPageViewModel(SearchPageViewModel const&) = delete;
        SearchPageViewModel& operator=(SearchPageViewModel const&) = delete;
        Windows::Foundation::Collections::IObservableVector<simbolmudah_ui::SequenceDetail> SearchResults() const;
        Windows::Foundation::IAsyncAction Search(hstring const& keyword);

    private:
        LibSimbolMudah::SequenceSearcher searcher;
        Windows::Foundation::Collections::IObservableVector<simbolmudah_ui::SequenceDetail> searchResults;
    };
}
namespace winrt::simbolmudah_ui::factory_implementation
{
    struct SearchPageViewModel : SearchPageViewModelT<SearchPageViewModel, implementation::SearchPageViewModel>
    {
    };
}

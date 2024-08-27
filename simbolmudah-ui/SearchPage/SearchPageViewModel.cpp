#include "pch.hpp"
#include "SearchPageViewModel.h"
#include "SearchPageViewModel.g.cpp"

namespace
{
    const winrt::simbolmudah_ui::SequenceDetail PLACEHOLDER{
        {}, L"🔍", L"Start typing to search", L"Results will show up as you type" };
    const winrt::simbolmudah_ui::SequenceDetail NO_RESULTS{
        {}, L"🤷", L"No results found", L"Try searching for something else" };
}

namespace winrt::simbolmudah_ui::implementation
{
    using namespace Windows::Foundation;
    using namespace Collections;
    using namespace LibSimbolMudah;

    SearchPageViewModel::SearchPageViewModel()
        : searchResults{ single_threaded_observable_vector<simbolmudah_ui::SequenceDetail>({ PLACEHOLDER }) } {}

    IObservableVector<simbolmudah_ui::SequenceDetail> SearchPageViewModel::SearchResults() const
    {
        return this->searchResults;
    }

    void SearchPageViewModel::SetSequenceDefinition(SequenceDefinition const& seqdef)
    {
        this->sequenceDefinition = seqdef;
        this->searchResults.ReplaceAll({ PLACEHOLDER });
    }

    IAsyncAction SearchPageViewModel::Search(hstring keyword)
    {
        if (keyword.empty())
        {
            this->searchResults.ReplaceAll({ PLACEHOLDER });
            co_return;
        }

        const auto ui_thread{ apartment_context() };
        co_await resume_background();
        const auto results{ this->sequenceDefinition.Search(keyword, 2000) };

        const auto size{ results.Size() };
        if (size == 0)
        {
            co_await ui_thread;
            this->searchResults.ReplaceAll({ NO_RESULTS });
            co_return;
        }

        std::vector<SequenceDetail> toShow;
        toShow.reserve(size);

        for (const auto& s: results)
        {
            toShow.emplace_back(s);
        }

        co_await ui_thread;
        this->searchResults.ReplaceAll(toShow);
    }
}

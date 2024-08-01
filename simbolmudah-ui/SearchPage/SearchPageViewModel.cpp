#include "pch.hpp"
#include "SearchPageViewModel.h"
#include "SearchPageViewModel.g.cpp"

namespace winrt::simbolmudah_ui::implementation
{
    using namespace Windows::Foundation;
    using namespace Collections;
    using namespace LibSimbolMudah;

    SearchPageViewModel::SearchPageViewModel(SequenceDefinition const& seqdef)
        : searchResults{ single_threaded_observable_vector<simbolmudah_ui::SequenceDetail>({
            simbolmudah_ui::SequenceDetail({L"`", L"e"}, L"🙏", L"tangan", L"U+XXXX"),
            simbolmudah_ui::SequenceDetail({L"`", L"a"}, L"🙏", L"tangan", L"U+XXXX")
        }) }, sequenceDefinition{ seqdef } {}

    IObservableVector<simbolmudah_ui::SequenceDetail> SearchPageViewModel::SearchResults() const
    {
        return this->searchResults;
    }

    IAsyncAction SearchPageViewModel::Search(hstring keyword)
    {
        const auto ui_thread{ apartment_context() };
        co_await resume_background();
        const auto results{ this->sequenceDefinition.Search(keyword, 2000) };

        const auto size{ results.Size() };
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

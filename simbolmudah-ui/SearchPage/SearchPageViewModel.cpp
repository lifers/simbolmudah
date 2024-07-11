#include "pch.h"
#include "SearchPageViewModel.h"
#include "SearchPageViewModel.g.cpp"

namespace winrt::simbolmudah_ui::implementation
{
	using namespace Windows::Foundation;
	using namespace Collections;
	using namespace LibSimbolMudah;

	SearchPageViewModel::SearchPageViewModel()
		: searchResults{ single_threaded_observable_vector<simbolmudah_ui::SequenceDetail>({
			simbolmudah_ui::SequenceDetail({1u, 2u, 3u}, L"🙏", L"tangan") 
		}) } {}

	IObservableVector<simbolmudah_ui::SequenceDetail> SearchPageViewModel::SearchResults() const
	{
		return this->searchResults;
	}

	IAsyncAction SearchPageViewModel::Search(hstring const& keyword)
	{
		const auto ui_thread{ apartment_context() };
		const auto str_copy{ keyword };
		co_await resume_background();

		com_array<IVectorView<uint32_t>> sequences;
		com_array<hstring> results;
		com_array<hstring> descriptions;
		this->searcher.Search(str_copy, sequences, results, descriptions);

		const auto size{ sequences.size() };
		assert(results.size() == size);
		assert(descriptions.size() == size);
		std::vector<SequenceDetail> toShow;
		toShow.reserve(size);

		for (auto i = 0u; i < size; ++i)
		{
			toShow.emplace_back(sequences[i], results[i], descriptions[i]);
		}
		
		co_await ui_thread;
		this->searchResults.ReplaceAll(toShow);
	}
}

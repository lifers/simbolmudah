#include "pch.hpp"
#include "SearchPageViewModel.h"
#include "SearchPageViewModel.g.cpp"
#include "App.xaml.h"

namespace
{
	using namespace winrt;
	using namespace Windows::Foundation::Collections;

	std::vector<UChar32> ToCodepoints(hstring const& str)
	{
		std::vector<UChar32> codepoints;
		UCharIterator iter;
		uiter_setString(&iter, str.c_str(), str.size());

		while (iter.hasNext(&iter))
		{
			codepoints.push_back(uiter_next32(&iter));
		}

		return codepoints;
	}

	IVectorView<hstring> CodepointsToStrings(std::vector<UChar32> cps)
	{
		std::vector<hstring> result;
		UChar buffer[2] = {};
		UErrorCode status{ U_ZERO_ERROR };
		int32_t written{ 0 };

		for (auto cp: cps)
		{
			if (u_strFromUTF32(buffer, 2, &written, &cp, 1, &status) != &buffer[0])
			{
				throw hresult_error(E_FAIL, L"u_strFromUTF32 failed");
			}
			else if (U_FAILURE(status))
			{
				throw hresult_error(ERROR_NO_UNICODE_TRANSLATION, L"Failed to convert codepoint to UTF-16");
			}
			result.emplace_back(buffer, written);
		}

		return single_threaded_vector<hstring>(std::move(result)).GetView();
	}

	hstring CodepointsToString(std::vector<UChar32> cps)
	{
		std::wstring result;

		for (auto cp: cps)
		{
			result.append(std::format(L"U+{:04X} ", cp));
		}

		return static_cast<hstring>(result);
	}
}

namespace winrt::simbolmudah_ui::implementation
{
	using namespace Windows::Foundation;
	using namespace Collections;
	using namespace LibSimbolMudah;
	using namespace Microsoft::UI::Xaml;

	SearchPageViewModel::SearchPageViewModel()
		: searchResults{ single_threaded_observable_vector<simbolmudah_ui::SequenceDetail>({
			simbolmudah_ui::SequenceDetail({L"`", L"e"}, L"🙏", L"tangan", L"U+XXXX"),
			simbolmudah_ui::SequenceDetail({L"`", L"a"}, L"🙏", L"tangan", L"U+XXXX")
		}) }, searcher{ Application::Current().as<App>()->sequenceDefinition } {}

	IObservableVector<simbolmudah_ui::SequenceDetail> SearchPageViewModel::SearchResults() const
	{
		return this->searchResults;
	}

	IAsyncAction SearchPageViewModel::Search(hstring const& keyword)
	{
		const auto ui_thread{ apartment_context() };
		const auto str_copy{ keyword };
		co_await resume_background();
		const auto results{ this->searcher.Search(str_copy) };

		const auto size{ results.Size() };
		std::vector<SequenceDetail> toShow;
		toShow.reserve(size);

		for (const auto s: results)
		{
			toShow.emplace_back(
				CodepointsToStrings(ToCodepoints(s.sequence)),
				s.result, s.description, CodepointsToString(ToCodepoints(s.result)));
		}
		
		co_await ui_thread;
		this->searchResults.ReplaceAll(toShow);
	}
}

#include "pch.h"
#include "SequenceDetail.h"
#include "SequenceDetail.g.cpp"

namespace
{
	using namespace winrt;

	hstring ToCodepointString(hstring const& str)
	{
		std::wstring result;
		UCharIterator iter;
		uiter_setString(&iter, str.c_str(), str.size());

		while (iter.hasNext(&iter))
		{
			const auto codepoint = uiter_next32(&iter);
			result.append(std::format(L"U+{:04X} ", codepoint));
		}

		return static_cast<hstring>(result);
	}
}

namespace winrt::simbolmudah_ui::implementation
{
	using namespace Microsoft::UI::Xaml::Data;
	using namespace Windows::Foundation::Collections;

    SequenceDetail::SequenceDetail(
		IVectorView<hstring> const& sequence,
		hstring const& result, hstring const& description, hstring const& codepoints) :
			m_sequence(single_threaded_observable_vector<hstring>(std::vector<hstring>(sequence.begin(), sequence.end()))),
			m_result(result), m_description(description), m_codepoints(codepoints) {}

	IObservableVector<hstring> SequenceDetail::Sequence() const { return this->m_sequence; }

    hstring SequenceDetail::Result() const { return this->m_result; }

    hstring SequenceDetail::Description() const { return this->m_description; }

	hstring SequenceDetail::Codepoints() const { return this->m_codepoints; }
}

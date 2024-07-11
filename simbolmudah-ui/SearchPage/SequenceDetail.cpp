#include "pch.h"
#include "SequenceDetail.h"
#include "SequenceDetail.g.cpp"

namespace winrt::simbolmudah_ui::implementation
{
	using namespace Microsoft::UI::Xaml::Data;
	using namespace Windows::Foundation::Collections;

    SequenceDetail::SequenceDetail(
		Windows::Foundation::Collections::IVectorView<uint32_t> const& sequence,
		hstring const& result, hstring const& description) :
			m_sequence(single_threaded_observable_vector<uint32_t>(std::vector<uint32_t>(sequence.begin(), sequence.end()))),
			m_result(result), m_description(description) {}

	IObservableVector<uint32_t> SequenceDetail::Sequence() const { return this->m_sequence; }

    hstring SequenceDetail::Result() const { return this->m_result; }

    hstring SequenceDetail::Description() const { return this->m_description; }
}

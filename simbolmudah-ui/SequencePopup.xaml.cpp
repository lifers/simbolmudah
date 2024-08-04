#include "pch.hpp"
#include "SequencePopup.xaml.h"
#if __has_include("SequencePopup.g.cpp")
#include "SequencePopup.g.cpp"
#endif
#include <wil/cppwinrt_helpers.h>

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Microsoft::UI::Xaml;
    using namespace Controls;
    using namespace Windows::Foundation;
    using namespace Collections;

    SequencePopup::SequencePopup(SequenceDefinition const& definition) :
        sequenceDefinition{ definition },
        sequence{ single_threaded_observable_vector<hstring>() },
        searchResults{ single_threaded_observable_vector<simbolmudah_ui::SequenceDetail>() } {}

    void SequencePopup::FindPotentialPrefix()
    {
        if (this->pendingSearch != nullptr) { this->pendingSearch.Cancel(); }
        this->pendingSearch = this->FindPotentialPrefixAsync();
        this->pendingSearch.Completed({ this->get_weak(), &SequencePopup::Find_Completed });
    }
    
    IObservableVector<hstring> SequencePopup::Sequence() const { return this->sequence; }

    IObservableVector<simbolmudah_ui::SequenceDetail> SequencePopup::SearchResults() const { return this->searchResults; }

    void SequencePopup::StackPanel_Loading(FrameworkElement const& sender, IInspectable const&) const
    {
        const auto& stackPanel{ sender.as<StackPanel>() };
        stackPanel.Translation({ 0.0f, 0.0f, 8.0f });
    }

    IAsyncAction SequencePopup::FindPotentialPrefixAsync()
    {
        std::wstring sequenceString;
        for (const auto& key : this->sequence) { sequenceString.append(key); }

        co_await resume_background();
        const auto results{ this->sequenceDefinition.PotentialPrefix(sequenceString, 1000) };

        std::vector<simbolmudah_ui::SequenceDetail> toShow;
        toShow.reserve(results.Size());
        for (const auto s: results) { toShow.emplace_back(s); }

        co_await wil::resume_foreground(this->DispatcherQueue());
        this->searchResults.ReplaceAll(toShow);
    }

    void SequencePopup::Find_Completed(IAsyncAction const&, AsyncStatus const&) { this->pendingSearch = nullptr; }
}

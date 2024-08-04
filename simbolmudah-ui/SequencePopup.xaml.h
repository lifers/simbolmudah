#pragma once

#include "SequencePopup.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct SequencePopup : SequencePopupT<SequencePopup>
    {
        explicit SequencePopup(const LibSimbolMudah::SequenceDefinition& definition);
        SequencePopup(const SequencePopup&) = delete;
        SequencePopup& operator=(const SequencePopup&) = delete;

        void FindPotentialPrefix();
        Windows::Foundation::Collections::IObservableVector<hstring> Sequence() const;
        Windows::Foundation::Collections::IObservableVector<simbolmudah_ui::SequenceDetail> SearchResults() const;

        void StackPanel_Loading(Microsoft::UI::Xaml::FrameworkElement const& sender, IInspectable const& args) const;

    private:
        Windows::Foundation::IAsyncAction FindPotentialPrefixAsync();
        void Find_Completed(Windows::Foundation::IAsyncAction const&, Windows::Foundation::AsyncStatus const&);

        const LibSimbolMudah::SequenceDefinition sequenceDefinition;
        const Windows::Foundation::Collections::IObservableVector<hstring> sequence;
        const Windows::Foundation::Collections::IObservableVector<simbolmudah_ui::SequenceDetail> searchResults;
        Windows::Foundation::IAsyncAction pendingSearch;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct SequencePopup : SequencePopupT<SequencePopup, implementation::SequencePopup>
    {
    };
}

#include "pch.hpp"
#include "SearchPage.xaml.h"
#include "SearchPage/SearchPageViewModel.h"
#if __has_include("SearchPage.g.cpp")
#include "SearchPage.g.cpp"
#endif

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Windows::Foundation;
    using namespace Microsoft::UI::Xaml;
    using namespace Controls;
    using namespace Navigation;

    simbolmudah_ui::SearchPageViewModel SearchPage::MainViewModel() const { return this->mainViewModel; }

    void SearchPage::SetSequenceDefinition(SequenceDefinition const& seqdef)
    {
        get_self<implementation::SearchPageViewModel>(this->mainViewModel)->SetSequenceDefinition(seqdef);
        this->SearchBox().IsEnabled(seqdef != nullptr);
    }

    void SearchPage::SubmitSearch(AutoSuggestBox const& sender, AutoSuggestBoxTextChangedEventArgs const&)
    {
        // Cancel the previous search if it is still running
        if (this->currentSearch) { this->currentSearch.Cancel(); }

        this->currentSearch = get_self<implementation::SearchPageViewModel>(this->mainViewModel)->Search(sender.Text());
        this->currentSearch.Completed({ this->get_weak(), &SearchPage::CurrentSearch_Completed });
    }

    void SearchPage::CurrentSearch_Completed(IAsyncAction const&, AsyncStatus) { this->currentSearch = nullptr; }
}

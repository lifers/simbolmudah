#include "pch.hpp"
#include "SearchPage.xaml.h"
#if __has_include("SearchPage.g.cpp")
#include "SearchPage.g.cpp"
#endif

namespace winrt::simbolmudah_ui::implementation
{
    using namespace Windows::Foundation;
    using namespace Microsoft::UI::Xaml;
    using namespace Controls;
    using namespace Navigation;

    simbolmudah_ui::SearchPageViewModel SearchPage::MainViewModel() const { return this->mainViewModel; }

    void SearchPage::SubmitSearch(AutoSuggestBox const& sender, AutoSuggestBoxTextChangedEventArgs const&)
    {
        // Cancel the previous search if it is still running
        if (this->currentSearch) { this->currentSearch.Cancel(); }

        this->currentSearch = this->mainViewModel.Search(sender.Text());
        this->currentSearch.Completed({ this->get_weak(), &SearchPage::CurrentSearch_Completed });
    }

    void SearchPage::OnNavigatedTo(NavigationEventArgs const& e)
    {
        const auto& seqdef{ e.Parameter().as<LibSimbolMudah::SequenceDefinition>() };
        this->mainViewModel = simbolmudah_ui::SearchPageViewModel{ seqdef };
    }

    void SearchPage::CurrentSearch_Completed(IAsyncAction const&, AsyncStatus) {
        
        if (this->mainViewModel.SearchResults().Size() == 0)
        {
            this->MainContent().Content(this->NoResultsView());
        }
        else
        {
            this->MainContent().Content(this->ResultsView());
        }
        this->currentSearch = nullptr;
    }
}

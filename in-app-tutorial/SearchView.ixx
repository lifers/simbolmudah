export module TutorialDialog:SearchView;

import pcm;

namespace tut
{
    using namespace winrt;
    using namespace Microsoft::UI::Xaml;
    using namespace Windows::Foundation;
    export Controls::StackPanel SearchView(
        ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup);
}
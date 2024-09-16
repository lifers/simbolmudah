export module TutorialDialog:NormalView;

import pcm;

namespace tut
{
    using namespace winrt;
    using namespace Microsoft::UI::Xaml;
    using namespace Windows::Foundation;
    export Controls::ScrollView NormalView(
        ResourceDictionary const& resCache, EventHandler<bool> const& hookPopup, bool& state);
}
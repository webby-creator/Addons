<script lang="ts">
    //
</script>

<div class="mt-3 space-y-4 max-w-[1248px] mx-auto">
    <!-- TOP BAR -->
    <div class="flex w-full">
        <div class="h-fit self-center">
            <button class="btn-icon btn-icon-sm variant-filled" type="button">
                <svg
                    viewBox="0 0 24 24"
                    fill="currentColor"
                    width="24px"
                    height="24px"
                    class="wds_1_104_2_Header__titleBackButtonIcon"
                    aria-hidden="true"
                >
                    <path
                        d="M14.8597212,8.14512634 C15.0549834,8.34038849 15.0549834,8.65697098 14.8597212,8.85223312 L11.2137854,12.5 L14.8597212,16.1465919 C15.0549834,16.3418541 15.0549834,16.6584366 14.8597212,16.8536987 C14.6644591,17.0489609 14.3478766,17.0489609 14.1526144,16.8536987 L9.79957183,12.5 L14.1526144,8.14512634 C14.3478766,7.9498642 14.6644591,7.9498642 14.8597212,8.14512634 Z"
                    />
                </svg>
            </button>
        </div>

        <div class="mr-auto px-3">
            <h2 class="h2">Blog Overview</h2>
            <p class="text-lg">
                Manage posts, track post performance and learn about new ways to
                improve your blog.
            </p>
        </div>

        <div class="flex h-fit space-x-1 self-center">
            <button class="btn variant-filled-secondary">More Actions</button>
            <button class="btn variant-filled" data-route="/create-post">
                Create New Post
            </button>
        </div>
    </div>

    <div class="space-y-8">
        <div
            class="w-full rounded-lg bg-white p-3 drop-shadow-xl divide-y divide-gray-200"
        >
            <div class="flex space-x-4 items-center px-2 pb-4">
                <div
                    aria-label="title"
                    class="flex mr-auto items-center space-x-4"
                >
                    <h4 class="h4">Analytics</h4>
                </div>
            </div>
            <div class="pt-4 pb-2 px-2 flex space-x-5">
                <!--  -->
            </div>
        </div>

        <div
            class="w-full rounded-lg bg-white p-3 drop-shadow-xl divide-y divide-gray-200"
        >
            <div class="flex space-x-4 items-center px-2 pb-4">
                <div aria-label="title" class="flex">
                    <h4 class="h4 grow">Last Published</h4>

                    <a href=";">Manage Posts</a>
                </div>
            </div>
            <div class="pt-4 pb-2 px-2 flex space-x-5">
                <div
                    class="flex w-full border-t border-slate-100 h-[52px] items-center space-x-2"
                >
                    <!-- Optional Image -->
                    <div class="w-[60px] h-[42px]">
                        <img
                            class="object-cover rounded"
                            width="60"
                            height="42"
                            alt="Img"
                            src="https://placehold.co/600x400"
                        />
                    </div>
                    <!-- Title -->
                    <div>
                        <p class="text-slate-800 text-lg">Test Title</p>
                    </div>
                    <!-- Date -->
                    <div>
                        <p class="text-slate-700">Jan 01, 2022</p>
                    </div>

                    <!-- Dropdown Button -->
                    <div class="!ml-auto">
                        <button class="btn btn-sm variant-filled">...</button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>

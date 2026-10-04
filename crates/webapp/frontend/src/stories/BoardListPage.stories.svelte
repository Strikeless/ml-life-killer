<script module>
    import { defineMeta } from "@storybook/addon-svelte-csf";
    import BoardListPage from "../lib/board_list/BoardListPage.svelte";

    const { Story } = defineMeta({
        component: BoardListPage,
        parameters: {
            boards: [
                {
                    id: "foo",
                    info: {
                        title: "Foo",
                        rules: "B2/S23",
                        width: 32,
                        height: 16,
                    },
                },
                {
                    id: "bar",
                    info: {
                        title: "Bar",
                        rules: "B23/S",
                        width: 1024,
                        height: 1024,
                    },
                },
            ],
            from_index: 0,
        },
    });
</script>

<Story name="BoardListPage" />

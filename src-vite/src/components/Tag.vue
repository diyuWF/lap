<template>
  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1">{{ localeMsg.tag.title }}</span>
      <div class="flex items-center gap-1">
        <TButton
          :icon="IconMore"
          :buttonSize="'small'"
          tooltip="管理标签分类体系"
          @click="showTaxonomyManager = true"
        />
        <TButton
          :icon="IconAdd"
          :buttonSize="'small'"
          :tooltip="$t('menu.tag.add')"
          @click="clickAddTag"
        />
      </div>
    </div>

    <div class="mx-1 mb-2 px-1 shrink-0">
      <div
        :class="[
          'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
          isTagSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
        ]"
      >
        <IconSearch class="ml-2 w-4 h-4 shrink-0" :class="isTagSearchFocused ? 'text-primary/70' : 'text-base-content/30'" />
        <input
          type="text"
          v-model="tagSearch"
          :placeholder="$t('tag.search_tags')"
          class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
          @focus="isTagSearchFocused = true"
          @blur="isTagSearchFocused = false"
        />
        <button
          v-if="tagSearch"
          type="button"
          class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70"
          @click="tagSearch = ''"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div class="grow overflow-x-hidden overflow-y-auto">
      <ul v-if="filteredTags.length > 0">
        <li v-for="tag in filteredTags" :key="tag.id" :id="'tag-' + tag.id">
          <div
            :class="[
              'sidebar-item group',
              selectedTag && selectedTag.id === tag.id && !isRenamingTag ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="selectTag(tag)"
            @contextmenu.prevent.stop="(e: MouseEvent) => handleTagContextMenu(tag, e)"
          >
            <IconTag class="mx-1 h-5 shrink-0" />
            <input v-if="selectedTag && selectedTag.id === tag.id && isRenamingTag"
              ref="tagInputRef"
              type="text"
              maxlength="255"
              class="input px-1 w-full focus:border text-base"
              v-model="tag.name"
              @keydown.enter="handleRenameTag"
              @keydown.esc="cancelRenameTag"
              @blur="handleRenameTag"
            />
            <span v-else class="sidebar-item-label">{{ tag.name }}</span>
            <span v-if="!isRenamingTag && tag.count" :class="['sidebar-item-count', selectedTag?.id === tag.id ? 'hidden' : 'group-hover:hidden']">{{ tag.count.toLocaleString() }}</span>
            <div
              v-if="!isRenamingTag"
              :class="['ml-auto flex flex-row items-center text-base-content/30', selectedTag?.id === tag.id ? '' : 'hidden group-hover:flex']"
            >
              <ContextMenu
                :ref="(el: any) => { if (el) tagContextMenus[tag.id] = el }"
                :iconMenu="IconMore"
                :menuItems="getMoreMenuItems()"
                :smallIcon="true"
              />
            </div>
          </div>
        </li>
      </ul>

      <div v-else-if="allTags.length > 0" class="sidebar-empty text-sm">
        <span class="text-center">{{ $t('tag.not_found') }}</span>
      </div>

      <div v-else class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
        <span class="text-sm text-center">{{ $t('tooltip.not_found.tag_hint') }}</span>
      </div>
    </div>
  </div>

  <MessageBox
    v-if="showNewTagMsgbox"
    :title="$t('msgbox.new_tag.title')"
    :showInput="true"
    :inputText="''"
    :inputPlaceholder="$t('msgbox.new_tag.content')"
    :needValidateInput="true"
    :OkText="$t('msgbox.new_tag.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickNewTag"
    @cancel="showNewTagMsgbox = false"
  />

  <MessageBox
    v-if="showDeleteTagMsgbox"
    :title="$t('msgbox.delete_tag.title')"
    :message="`${$t('msgbox.delete_tag.content', { tag: selectedTag.name })}`"
    :OkText="$t('msgbox.delete_tag.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clickDeleteTag"
    @cancel="showDeleteTagMsgbox = false"
  />

  <TaxonomyManager
    v-if="showTaxonomyManager"
    @close="showTaxonomyManager = false"
    @changed="loadTags"
  />
</template>

<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { getAllTags, renameTag, deleteTag, createTag } from '@/common/api';
import {
  IconAdd,
  IconClose,
  IconMore,
  IconSearch,
  IconTag,
  IconRename,
  IconTrash,
} from '@/common/icons';

import ContextMenu from '@/components/ContextMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';
import TaxonomyManager from '@/components/TaxonomyManager.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const emit = defineEmits(['editDataChanged']);

const allTags = ref<any[]>([]);
const selectedTag = ref<any>(null);
const isRenamingTag = ref(false);
const originalTagName = ref('');
const tagInputRef = ref<HTMLInputElement[]>([]);
const tagSearch = ref('');
const isTagSearchFocused = ref(false);
const showTaxonomyManager = ref(false);

const sortedTags = computed(() => allTags.value);
const filteredTags = computed(() => {
  const query = tagSearch.value.trim().toLowerCase();
  if (!query) return sortedTags.value;
  return sortedTags.value.filter(tag => tag.name.toLowerCase().includes(query));
});

const showDeleteTagMsgbox = ref(false);
const showNewTagMsgbox = ref(false);
const tagContextMenus = ref<Record<number, any>>({});

function handleTagContextMenu(tag: any, event: MouseEvent) {
  selectTag(tag);
  tagContextMenus.value[tag.id]?.open?.(event.clientX, event.clientY);
}

const getMoreMenuItems = () => [
  {
    label: localeMsg.value.menu.tag.rename,
    icon: IconRename,
    action: () => {
      isRenamingTag.value = true;
      originalTagName.value = selectedTag.value.name;
      nextTick(() => {
        if (tagInputRef.value) tagInputRef.value[0].focus();
      });
    }
  },
  {
    label: localeMsg.value.menu.tag.delete,
    icon: IconTrash,
    action: () => {
      showDeleteTagMsgbox.value = true;
    },
  },
];

onMounted(() => {
  loadTags();
});

watch(() => config.settings.categorySort, () => {
  loadTags();
});

async function loadTags() {
  const tags = await getAllTags(config.settings.categorySort);
  if (tags) {
    allTags.value = tags;
    if (allTags.value.length > 0) {
      const index = allTags.value.findIndex(tag => tag.id === libConfig.tag.id);
      if (index >= 0) {
        selectedTag.value = allTags.value[index];
      } else if (!selectedTag.value) {
        selectedTag.value = allTags.value[0];
        libConfig.tag.id = selectedTag.value.id;
      }
    }
  } else {
    libConfig.tag.id = null;
    selectedTag.value = null;
  }
}

function selectTag(tag: any) {
  if (isRenamingTag.value) return;
  selectedTag.value = tag;
  libConfig.tag.id = tag.id;
}

async function handleRenameTag() {
  if (!isRenamingTag.value) return;
  const newName = selectedTag.value.name.trim();
  if (newName.length === 0 || newName === originalTagName.value) {
    isRenamingTag.value = false;
    selectedTag.value.name = originalTagName.value;
    return;
  }
  const result = await renameTag(selectedTag.value.id, newName);
  if (result) isRenamingTag.value = false;
}

function cancelRenameTag() {
  selectedTag.value.name = originalTagName.value;
  isRenamingTag.value = false;
}

function clickAddTag() {
  showNewTagMsgbox.value = true;
}

async function clickNewTag(newTagName: string) {
  if (!newTagName || newTagName.trim().length === 0) return;
  const result = await createTag(newTagName);
  if (result) {
    showNewTagMsgbox.value = false;
    await loadTags();
    const newTag = allTags.value.find(tag => tag.name === newTagName);
    if (newTag) {
      selectTag(newTag);
      nextTick(() => scrollToTag(newTag.id));
    }
  }
}

function scrollToTag(tagId: number) {
  const tagElement = document.getElementById(`tag-${tagId}`);
  if (tagElement) tagElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}

async function clickDeleteTag() {
  if (!selectedTag.value) return;
  showDeleteTagMsgbox.value = false;
  const result = await deleteTag(selectedTag.value.id);
  if (!result) return;
  const index = allTags.value.findIndex(tag => tag.id === selectedTag.value.id);
  allTags.value = allTags.value.filter(tag => tag.id !== selectedTag.value.id);
  if (index > 0) {
    selectTag(allTags.value[index - 1]);
  } else if (index === 0 && allTags.value.length > 0) {
    selectTag(allTags.value[0]);
  } else {
    selectedTag.value = null;
    libConfig.tag.id = null;
  }
}

defineExpose({ clickAddTag });
</script>

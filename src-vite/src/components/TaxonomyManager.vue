<template>
  <div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/55 p-4" @mousedown.self="$emit('close')">
    <section class="flex h-[78vh] w-[900px] max-w-[96vw] overflow-hidden rounded-box border border-base-content/10 bg-base-200 shadow-2xl">
      <aside class="w-64 shrink-0 border-r border-base-content/10 bg-base-300/60 p-3">
        <div class="mb-3 flex items-center justify-between">
          <div>
            <h2 class="font-semibold">标签分类体系</h2>
            <p class="text-xs text-base-content/45">分组、层级与别名</p>
          </div>
          <button class="btn btn-ghost btn-sm" type="button" @click="$emit('close')">关闭</button>
        </div>

        <button class="btn btn-primary btn-sm mb-3 w-full" type="button" @click="startNewGroup">新建分组</button>
        <div class="space-y-1 overflow-y-auto">
          <button
            type="button"
            class="flex w-full items-center gap-2 rounded-box px-3 py-2 text-left text-sm"
            :class="selectedGroupId === null ? 'bg-primary text-primary-content' : 'hover:bg-base-100/50'"
            @click="selectGroup(null)"
          >
            <span class="h-3 w-3 rounded-full border border-current/30"></span>
            <span class="min-w-0 flex-1 truncate">未分组</span>
            <span class="text-xs opacity-60">{{ ungroupedCount }}</span>
          </button>
          <button
            v-for="group in groups"
            :key="group.id"
            type="button"
            class="flex w-full items-center gap-2 rounded-box px-3 py-2 text-left text-sm"
            :class="selectedGroupId === group.id ? 'bg-primary text-primary-content' : 'hover:bg-base-100/50'"
            @click="selectGroup(group.id)"
          >
            <span class="h-3 w-3 rounded-full border border-black/15" :style="{ backgroundColor: group.color || '#9097a6' }"></span>
            <span class="min-w-0 flex-1 truncate">{{ group.name }}</span>
            <span class="text-xs opacity-60">{{ group.tagCount }}</span>
          </button>
        </div>
      </aside>

      <main class="flex min-w-0 flex-1 flex-col">
        <header class="flex items-center gap-2 border-b border-base-content/10 p-3">
          <input v-model="search" class="input input-bordered input-sm flex-1" type="search" placeholder="搜索标签或别名" />
          <button v-if="activeGroup" class="btn btn-ghost btn-sm" type="button" @click="editCurrentGroup">编辑分组</button>
          <button v-if="activeGroup" class="btn btn-ghost btn-sm text-error" type="button" @click="removeCurrentGroup">删除分组</button>
        </header>

        <div class="grid min-h-0 flex-1 grid-cols-[minmax(250px,0.9fr)_minmax(330px,1.1fr)]">
          <section class="overflow-y-auto border-r border-base-content/10 p-3">
            <div v-if="loading" class="py-10 text-center text-sm text-base-content/45">正在加载分类体系…</div>
            <div v-else-if="filteredTags.length === 0" class="py-10 text-center text-sm text-base-content/45">当前分组没有标签</div>
            <button
              v-for="tag in filteredTags"
              :key="tag.id"
              type="button"
              class="mb-1 flex w-full items-center gap-2 rounded-box px-3 py-2 text-left"
              :class="selectedTag?.id === tag.id ? 'bg-base-100 text-primary' : 'hover:bg-base-100/50'"
              @click="selectTag(tag)"
            >
              <span class="h-3 w-3 rounded-full border border-black/15" :style="{ backgroundColor: tag.color || activeGroup?.color || '#9097a6' }"></span>
              <span class="min-w-0 flex-1">
                <span class="block truncate text-sm font-medium">{{ tag.name }}</span>
                <span class="block truncate text-[11px] text-base-content/40">
                  {{ parentName(tag.parentId) || '顶级标签' }}<template v-if="tag.aliases.length"> · {{ tag.aliases.join('、') }}</template>
                </span>
              </span>
              <span class="text-xs text-base-content/35">{{ tag.fileCount }}</span>
            </button>
          </section>

          <section class="overflow-y-auto p-4">
            <div v-if="selectedTag" class="space-y-4">
              <div>
                <h3 class="text-lg font-semibold">{{ selectedTag.name }}</h3>
                <p class="text-xs text-base-content/45">设置分类分组、父子关系、颜色和检索别名。</p>
              </div>

              <label class="form-control gap-1">
                <span class="text-xs text-base-content/60">所属分组</span>
                <select v-model="tagForm.groupId" class="select select-bordered select-sm">
                  <option :value="null">未分组</option>
                  <option v-for="group in groups" :key="group.id" :value="group.id">{{ group.name }}</option>
                </select>
              </label>

              <label class="form-control gap-1">
                <span class="text-xs text-base-content/60">父级标签</span>
                <select v-model="tagForm.parentId" class="select select-bordered select-sm">
                  <option :value="null">无（顶级标签）</option>
                  <option v-for="tag in parentCandidates" :key="tag.id" :value="tag.id">{{ tag.name }}</option>
                </select>
              </label>

              <label class="form-control gap-1">
                <span class="text-xs text-base-content/60">标签颜色</span>
                <div class="flex gap-2">
                  <input v-model="tagForm.color" class="input input-bordered input-sm flex-1 font-mono" type="text" placeholder="#6388FF" />
                  <input v-model="tagForm.color" class="h-8 w-12 cursor-pointer rounded border-0 bg-transparent" type="color" />
                </div>
              </label>

              <label class="form-control gap-1">
                <span class="text-xs text-base-content/60">别名／同义词</span>
                <textarea v-model="tagForm.aliases" class="textarea textarea-bordered min-h-20" placeholder="每行一个别名，也可使用逗号分隔"></textarea>
                <span class="text-[11px] text-base-content/40">例如“汽车”“automobile”“vehicle”可指向同一个正式标签。</span>
              </label>

              <label class="form-control gap-1">
                <span class="text-xs text-base-content/60">说明</span>
                <textarea v-model="tagForm.description" class="textarea textarea-bordered min-h-24" placeholder="说明该标签的适用范围，供人工和 AI 整理时参考"></textarea>
              </label>

              <div v-if="error" class="alert alert-error py-2 text-sm">{{ error }}</div>
              <div class="flex justify-end gap-2">
                <button class="btn btn-ghost btn-sm" type="button" @click="resetTagForm">撤销修改</button>
                <button class="btn btn-primary btn-sm" type="button" :disabled="saving" @click="saveTag">{{ saving ? '正在保存…' : '保存标签设置' }}</button>
              </div>
            </div>
            <div v-else class="flex h-full items-center justify-center text-sm text-base-content/40">从左侧选择一个标签进行编辑</div>
          </section>
        </div>
      </main>
    </section>

    <div v-if="showGroupEditor" class="fixed inset-0 z-[110] flex items-center justify-center bg-black/45" @mousedown.self="showGroupEditor = false">
      <form class="w-96 max-w-[92vw] space-y-4 rounded-box bg-base-100 p-5 shadow-2xl" @submit.prevent="saveGroup">
        <h3 class="text-lg font-semibold">{{ groupForm.id ? '编辑标签分组' : '新建标签分组' }}</h3>
        <label class="form-control gap-1">
          <span class="text-xs text-base-content/60">分组名称</span>
          <input v-model="groupForm.name" class="input input-bordered" maxlength="80" required />
        </label>
        <label class="form-control gap-1">
          <span class="text-xs text-base-content/60">分组颜色</span>
          <div class="flex gap-2">
            <input v-model="groupForm.color" class="input input-bordered flex-1 font-mono" placeholder="#6388FF" />
            <input v-model="groupForm.color" class="h-12 w-14 cursor-pointer rounded border-0 bg-transparent" type="color" />
          </div>
        </label>
        <div v-if="groupError" class="alert alert-error py-2 text-sm">{{ groupError }}</div>
        <div class="flex justify-end gap-2">
          <button class="btn btn-ghost" type="button" @click="showGroupEditor = false">取消</button>
          <button class="btn btn-primary" type="submit">保存</button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import {
  deleteTaxonomyGroup,
  getTaxonomySnapshot,
  saveTaxonomyGroup,
  saveTaxonomyTag,
} from '@/common/dam-api';

const emit = defineEmits(['close', 'changed']);
const loading = ref(true);
const saving = ref(false);
const error = ref('');
const groupError = ref('');
const search = ref('');
const groups = ref<any[]>([]);
const tags = ref<any[]>([]);
const selectedGroupId = ref<number | null>(null);
const selectedTag = ref<any>(null);
const showGroupEditor = ref(false);
const groupForm = reactive({ id: null as number | null, name: '', color: '#6388FF' });
const tagForm = reactive({ groupId: null as number | null, parentId: null as number | null, color: '#6388FF', aliases: '', description: '' });

const activeGroup = computed(() => groups.value.find((group) => group.id === selectedGroupId.value) || null);
const ungroupedCount = computed(() => tags.value.filter((tag) => tag.groupId == null).length);
const filteredTags = computed(() => {
  const query = search.value.trim().toLowerCase();
  return tags.value
    .filter((tag) => tag.groupId === selectedGroupId.value)
    .filter((tag) => !query || tag.name.toLowerCase().includes(query) || tag.aliases.some((alias: string) => alias.toLowerCase().includes(query)));
});
const parentCandidates = computed(() => tags.value.filter((tag) => tag.id !== selectedTag.value?.id));

function parentName(parentId: number | null) {
  if (!parentId) return '';
  return tags.value.find((tag) => tag.id === parentId)?.name || '';
}

async function load() {
  loading.value = true;
  try {
    const snapshot = await getTaxonomySnapshot();
    groups.value = snapshot.groups || [];
    tags.value = snapshot.tags || [];
    if (selectedGroupId.value !== null && !groups.value.some((group) => group.id === selectedGroupId.value)) selectedGroupId.value = null;
    if (selectedTag.value) {
      selectedTag.value = tags.value.find((tag) => tag.id === selectedTag.value.id) || null;
      if (selectedTag.value) resetTagForm();
    }
  } finally {
    loading.value = false;
  }
}

function selectGroup(groupId: number | null) {
  selectedGroupId.value = groupId;
  selectedTag.value = null;
}

function selectTag(tag: any) {
  selectedTag.value = tag;
  resetTagForm();
}

function resetTagForm() {
  if (!selectedTag.value) return;
  tagForm.groupId = selectedTag.value.groupId ?? null;
  tagForm.parentId = selectedTag.value.parentId ?? null;
  tagForm.color = selectedTag.value.color || activeGroup.value?.color || '#6388FF';
  tagForm.aliases = (selectedTag.value.aliases || []).join('\n');
  tagForm.description = selectedTag.value.description || '';
  error.value = '';
}

function parseAliases(value: string) {
  return value.split(/[\n,，]/).map((item) => item.trim()).filter(Boolean);
}

async function saveTag() {
  if (!selectedTag.value || saving.value) return;
  saving.value = true;
  error.value = '';
  try {
    await saveTaxonomyTag({
      tagId: selectedTag.value.id,
      groupId: tagForm.groupId,
      parentId: tagForm.parentId,
      color: tagForm.color,
      aliases: parseAliases(tagForm.aliases),
      description: tagForm.description,
    });
    selectedGroupId.value = tagForm.groupId;
    await load();
    emit('changed');
  } catch (cause: any) {
    error.value = String(cause);
  } finally {
    saving.value = false;
  }
}

function startNewGroup() {
  Object.assign(groupForm, { id: null, name: '', color: '#6388FF' });
  groupError.value = '';
  showGroupEditor.value = true;
}

function editCurrentGroup() {
  if (!activeGroup.value) return;
  Object.assign(groupForm, { id: activeGroup.value.id, name: activeGroup.value.name, color: activeGroup.value.color || '#6388FF' });
  groupError.value = '';
  showGroupEditor.value = true;
}

async function saveGroup() {
  groupError.value = '';
  try {
    const saved = await saveTaxonomyGroup({ ...groupForm });
    selectedGroupId.value = saved.id;
    showGroupEditor.value = false;
    await load();
    emit('changed');
  } catch (cause: any) {
    groupError.value = String(cause);
  }
}

async function removeCurrentGroup() {
  if (!activeGroup.value) return;
  if (!confirm(`删除分组“${activeGroup.value.name}”？其中标签将变为未分组。`)) return;
  await deleteTaxonomyGroup(activeGroup.value.id, null);
  selectedGroupId.value = null;
  selectedTag.value = null;
  await load();
  emit('changed');
}

onMounted(load);
</script>

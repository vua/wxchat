<script setup lang="ts">
import {globalStore, OpenAiConfig} from "../stores/global_store.ts";
import {storeToRefs} from "pinia";
import {convertFileSrc, invoke} from "@tauri-apps/api/core";
import {Setting} from "@element-plus/icons-vue";

import {basicSetup, EditorView} from "codemirror"
import {json} from "@codemirror/lang-json"
import {ref, watch} from "vue";
import {oneDark} from "@codemirror/theme-one-dark"
import {ElMessage} from "element-plus";


const store = globalStore()
const {user, root, openai_config_list} = storeToRefs(store)

const clickUserAvatar = () => {

}

const openai_config_editor = ref<Element>()

// onMounted(() => {
//   nextTick(() => {
//     if (openai_config_editor.value && openai_system_config.value) {
//       new EditorView({
//         doc: "hi!",
//         extensions: [basicSetup],
//         parent: openai_config_editor.value
//       })
//     }
//   })
// })

const drawer = ref(false)
const openai_system_config = ref(false)

let editor = ref<EditorView | null>(null);

watch(openai_config_editor, (_) => {
  if (!openai_config_editor.value) {
    return;
  }
  if (!editor.value) {
    editor.value = new EditorView({
      doc: JSON.stringify(openai_config_list.value, null, 4),
      extensions: [basicSetup, oneDark, json()],
      parent: openai_config_editor.value
    })
  }
})

const getImgSrc = (name?: string) => {
  return convertFileSrc(`${root.value}/resources/image/avatar/${name}.jpg`)
}

async function updateOpenAiConfig() {
  if (editor.value) {
    let configs = editor.value.state.doc.toString()
    try {
      console.log("OpenAiConfig:", JSON.parse(configs) as OpenAiConfig[])
    } catch (e) {
      ElMessage(
          {
            message: "OpenAiConfig , 更新失败 , 请检测配置格式.",
            type: "warning"
          }
      )
      return
    }

    openai_config_list.value = JSON.parse(await invoke("update_openai_config", {configs: configs})) as OpenAiConfig[]
    ElMessage(
        {
          message: "OpenAiConfig , 更新成功 .",
          type: "success"
        }
    )
  }
}

</script>

<template>
  <div class="user-container">
    <div class="avatar-container">
      <img class="avatar"
           :src="getImgSrc(user?.UserName)"
           alt="" @click="clickUserAvatar">
    </div>
    <div class="setting-container">
      <el-icon style="font-size: 30px;color: #6b778c" @click="drawer=true">
        <Setting/>
      </el-icon>

    </div>
    <el-drawer v-model="drawer" size="20%" title="系统设置">
      <div class="setting-list">
        <el-button type="primary" @click="openai_system_config = true" style="width: 100%" plain>
          大模型配置
        </el-button>
        <el-drawer
            v-model="openai_system_config"
            size="50%"
            title="大模型配置"
            :append-to-body="true"
        >

          <div style="padding: 5px 0">
            <el-button type="primary" @click="updateOpenAiConfig">
              更新
            </el-button>
          </div>
          <div ref="openai_config_editor" id="openai-config-editor"></div>

        </el-drawer>
      </div>
    </el-drawer>
  </div>
</template>

<style scoped>
.user-container {
  background: rgb(214, 213, 206);
  display: flex;
  justify-content: center;
  width: 60px;
  height: 100vh;
  flex-direction: column;
}

.avatar {
  width: 40px;
  height: 40px;
  border-radius: 5px;
}

.avatar-container {
  padding-top: 20px;
  display: flex;
  align-items: center;
  flex-direction: column;
  height: 50vh;
}

.setting-container {
  display: flex;
  align-items: center;
  flex-direction: column-reverse;
  padding-bottom: 20px;
  height: 50vh;
}

.setting-list {
  display: flex;
  align-items: center;
  flex-direction: column;
  width: 100%;
}

#openai-config-editor {
  width: 100%;
}
</style>
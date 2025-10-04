<template>
  <div>
    <div class="toolbar">
      <el-button type="primary" :icon="Plus" @click="handleAdd">Add Group</el-button>
    </div>

    <el-table :data="groups" border style="width: 100%">
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="app_name" label="App Name" />
      <el-table-column prop="title" label="Title" />
      <el-table-column label="Address Type">
        <template #default="{ row }">
          {{ row.address_type === 0 ? 'Auto' : 'Manual' }}
        </template>
      </el-table-column>
      <el-table-column prop="address_list" label="Address List" />
      <el-table-column label="Actions" width="200">
        <template #default="{ row }">
          <el-button size="small" @click="handleEdit(row)">Edit</el-button>
          <el-button size="small" type="danger" @click="handleDelete(row.id)">Delete</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="50%">
      <el-form :model="form" label-width="120px">
        <el-form-item label="App Name">
          <el-input v-model="form.app_name" />
        </el-form-item>
        <el-form-item label="Title">
          <el-input v-model="form.title" />
        </el-form-item>
        <el-form-item label="Address Type">
          <el-radio-group v-model="form.address_type">
            <el-radio :label="0">Auto</el-radio>
            <el-radio :label="1">Manual</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item v-if="form.address_type === 1" label="Address List">
          <el-input v-model="form.address_list" type="textarea" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">Cancel</el-button>
        <el-button type="primary" @click="handleSave">Save</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { Plus } from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'

const groups = ref([])
const dialogVisible = ref(false)
const dialogTitle = ref('')
const form = ref({})

const fetchGroups = async () => {
  try {
    const response = await api.getJobGroups()
    groups.value = response.data
  } catch (error) {
    ElMessage.error('Failed to fetch job groups.')
  }
}

onMounted(fetchGroups)

const getNewForm = () => ({
  id: null,
  app_name: '',
  title: '',
  address_type: 0,
  address_list: '',
})

const handleAdd = () => {
  form.value = getNewForm()
  dialogTitle.value = 'Add Group'
  dialogVisible.value = true
}

const handleEdit = (row) => {
  form.value = { ...row }
  dialogTitle.value = 'Edit Group'
  dialogVisible.value = true
}

const handleSave = async () => {
  try {
    if (form.value.id) {
      await api.updateJobGroup(form.value)
      ElMessage.success('Group updated successfully.')
    } else {
      await api.addJobGroup(form.value)
      ElMessage.success('Group added successfully.')
    }
    dialogVisible.value = false
    await fetchGroups()
  } catch (error) {
    ElMessage.error('Failed to save group.')
  }
}

const handleDelete = (id) => {
  ElMessageBox.confirm('Are you sure you want to delete this group?', 'Warning', {
    confirmButtonText: 'OK',
    cancelButtonText: 'Cancel',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.deleteJobGroup(id)
        ElMessage.success('Group deleted successfully.')
        await fetchGroups()
      } catch (error) {
        ElMessage.error('Failed to delete group.')
      }
    })
    .catch(() => {
      // Canceled
    })
}
</script>

<style scoped>
.toolbar {
  margin-bottom: 20px;
}
</style>
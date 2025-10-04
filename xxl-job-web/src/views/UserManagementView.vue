<template>
  <div>
    <div class="toolbar">
      <el-button type="primary" :icon="Plus" @click="handleAdd">Add User</el-button>
    </div>

    <el-table :data="users" border style="width: 100%">
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="username" label="Username" />
      <el-table-column label="Role">
        <template #default="{ row }">
          {{ row.role === 1 ? 'Admin' : 'User' }}
        </template>
      </el-table-column>
      <el-table-column label="Actions" width="200">
        <template #default="{ row }">
          <el-button size="small" @click="handleEdit(row)">Edit</el-button>
          <el-button size="small" type="danger" @click="handleDelete(row.id)">Delete</el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="50%">
      <el-form :model="form" label-width="120px">
        <el-form-item label="Username">
          <el-input v-model="form.username" />
        </el-form-item>
        <el-form-item label="Password">
          <el-input v-model="form.password" type="password" placeholder="Leave blank to keep current password" />
        </el-form-item>
        <el-form-item label="Role">
          <el-radio-group v-model="form.role">
            <el-radio :label="1">Admin</el-radio>
            <el-radio :label="0">User</el-radio>
          </el-radio-group>
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

const users = ref([])
const dialogVisible = ref(false)
const dialogTitle = ref('')
const form = ref({})

const fetchUsers = async () => {
  try {
    const response = await api.getUsers()
    users.value = response.data
  } catch (error) {
    ElMessage.error('Failed to fetch users.')
  }
}

onMounted(fetchUsers)

const getNewForm = () => ({
  id: null,
  username: '',
  password: '',
  role: 0,
  permission: '',
})

const handleAdd = () => {
  form.value = getNewForm()
  dialogTitle.value = 'Add User'
  dialogVisible.value = true
}

const handleEdit = (row) => {
  form.value = { ...row, password: '' } // Clear password for editing
  dialogTitle.value = 'Edit User'
  dialogVisible.value = true
}

const handleSave = async () => {
  try {
    if (form.value.id) {
      await api.updateUser(form.value)
      ElMessage.success('User updated successfully.')
    } else {
      await api.addUser(form.value)
      ElMessage.success('User added successfully.')
    }
    dialogVisible.value = false
    await fetchUsers()
  } catch (error) {
    ElMessage.error('Failed to save user.')
  }
}

const handleDelete = (id) => {
  ElMessageBox.confirm('Are you sure you want to delete this user?', 'Warning', {
    confirmButtonText: 'OK',
    cancelButtonText: 'Cancel',
    type: 'warning',
  })
    .then(async () => {
      try {
        await api.deleteUser(id)
        ElMessage.success('User deleted successfully.')
        await fetchUsers()
      } catch (error) {
        ElMessage.error('Failed to delete user.')
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